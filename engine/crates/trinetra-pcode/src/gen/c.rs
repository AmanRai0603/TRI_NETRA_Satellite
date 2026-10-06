//! The translator to C: design/js/pcode_c.js `toC`, line for line. C99, no dynamic memory, no
//! recursion, in the flight software's conventions (fsw/pseudocode/00_conventions.md): every array a
//! small struct passed and returned by value, every function and table prefixed with its module, a
//! proc's state a struct it is handed, and the interpreter's arithmetic. Each array shape and record
//! is declared once, before what holds it, and each array operation once per shape it is used at,
//! in the order the translation first meets them, as the JavaScript's maps keep them.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

use super::*;
use crate::ast::{BinOp, ExprKind, FnKind, Stmt, StmtKind, TableMode};
use crate::check::{ItemRef, Shape, Target, VarKind};

/// words a name of the pseudocode may not be in C: its keywords, what <math.h>, <stdint.h> and <stdbool.h>
/// define as macros or functions we call, and the names the translation uses itself
const RESERVED: &[&str] = &[
    "auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else", "enum", "extern", "float", "for", "goto", "if", "inline",
    "int", "long", "register", "restrict", "return", "short", "signed", "sizeof", "static", "struct", "switch", "typedef", "union", "unsigned",
    "void", "volatile", "while", "_Bool", "_Complex", "_Imaginary", "bool", "true", "false", "NULL", "errno", "assert", "isnan", "isinf",
    "isfinite", "signbit", "fpclassify", "isnormal", "NAN", "INFINITY", "HUGE_VAL", "M_PI", "sqrt", "sin", "cos", "tan", "asin", "acos", "atan",
    "atan2", "exp", "log", "log10", "pow", "floor", "ceil", "round", "fmod", "fabs", "hypot", "llabs", "int8_t", "int16_t", "int32_t", "int64_t",
    "uint8_t", "uint16_t", "uint32_t", "uint64_t", "size_t", "st", "main",
];

fn cname(n: &str) -> String {
    if RESERVED.contains(&n) {
        format!("{n}_")
    } else {
        n.to_string()
    }
}

fn mode_text(m: TableMode) -> &'static str {
    match m {
        TableMode::Step => "step",
        TableMode::Linear => "linear",
    }
}

/// a C comment's text: no `*/` inside
fn no_close(s: &str) -> String {
    s.replace("*/", "* /")
}

fn doc(lines: &[String]) -> String {
    lines.iter().map(|l| format!("{}\n", trim_end_js(&format!("/* {} */", no_close(l))))).collect()
}

/// the include guard of a library's name: upper case, every other character `_`
fn guard_of(lib: &str) -> String {
    upper(lib).chars().map(|ch| if ch.is_ascii_uppercase() || ch.is_ascii_digit() { ch.to_string() } else { "_".repeat(ch.len_utf16()) }).collect()
}

pub(super) fn to_c(i: &Interp, title: Option<&str>, lib: Option<&str>) -> Result<Files, String> {
    let lib = lib.filter(|l| !l.is_empty()).unwrap_or("pcode").to_string();
    let mut g = Gen { b: Base::new(i), types: Vec::new(), helpers: Vec::new(), tmp: 0 };
    let files = g.run(title, &lib);
    g.b.done(files)
}

struct Gen<'a> {
    b: Base<'a>,
    /// C name -> definition, in dependency order (None: a record's place, reserved while its fields are declared)
    types: Vec<(String, Option<String>)>,
    /// the runtime's array operations, each once
    helpers: Vec<(String, String)>,
    tmp: usize,
}

/// a type made from its parts, as the JavaScript makes `{ k: "arr", n, of }`
fn arr(n: usize, of: Ty) -> Ty {
    Ty::Arr(n, Box::new(of))
}

impl<'a> Gen<'a> {
    fn has_type(&self, n: &str) -> bool {
        self.types.iter().any(|(k, _)| k == n)
    }
    fn tag(&mut self, t: &Ty) -> String {
        match t {
            Ty::Real(_) => "f".into(),
            Ty::Int => "i".into(),
            Ty::Bool => "b".into(),
            Ty::Arr(n, of) => format!("a{n}{}", self.tag(of)),
            Ty::Rec(name) => format!("r{name}"),
            Ty::Tuple(_) => self.b.fail("no C type for tuple"),
        }
    }
    fn cty(&mut self, t: &Ty) -> String {
        match t {
            Ty::Real(_) => "double".into(),
            Ty::Int => "int64_t".into(),
            Ty::Bool => "bool".into(),
            Ty::Rec(name) => {
                self.record_type(name);
                name.clone()
            }
            Ty::Arr(n, of) => {
                let name = format!("pc_{}", self.tag(t));
                if !self.has_type(&name) {
                    let el = self.cty(of);
                    self.types.push((name.clone(), Some(format!("typedef struct {{ {el} v[{n}]; }} {name};"))));
                }
                name
            }
            Ty::Tuple(_) => self.b.fail("no C type for tuple"),
        }
    }
    fn record_type(&mut self, name: &str) {
        if self.has_type(name) {
            return;
        }
        let Some(r) = self.b.record(name) else {
            self.b.fail(format!("cannot read properties of undefined (reading 'fields'): no record {name}"));
            return;
        };
        self.types.push((name.to_string(), None)); // reserve the place: fields first, then the record
        let fields: String = r.fields.iter().map(|f| format!("    {} {};\n", self.cty(&f.ty), cname(&f.name))).collect();
        self.types.retain(|(k, _)| k != name);
        self.types.push((name.to_string(), Some(format!("typedef struct {{\n{fields}}} {name};"))));
    }
    fn zero(&mut self, t: &Ty) -> String {
        match t {
            Ty::Real(_) => "0.0".into(),
            Ty::Int => "0".into(),
            Ty::Bool => "false".into(),
            _ => format!("(({}){{0}})", self.cty(t)),
        }
    }
    // a value as an initializer (file-scope constants, a proc's starting state)
    fn init(&mut self, v: &Value, t: &Ty) -> String {
        match t {
            Ty::Arr(_, of) => match v {
                Value::Arr(items) => {
                    let xs: Vec<String> = items.iter().map(|x| self.init(x, of)).collect();
                    format!("{{ {{ {} }} }}", xs.join(", "))
                }
                _ => self.b.fail("v.map is not a function"),
            },
            Ty::Rec(name) => {
                let Some(r) = self.b.record(name) else { return self.b.fail(format!("no record {name}")) };
                let Value::Rec(vals) = v else { return self.b.fail("a record's value is not a record") };
                let xs: Vec<String> = r
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| match vals.get(i) {
                        Some(x) => self.init(x, &f.ty),
                        None => self.b.fail("a record's value lacks a field"),
                    })
                    .collect();
                format!("{{ {} }}", xs.join(", "))
            }
            Ty::Bool => match v {
                Value::Bool(b) => b.to_string(),
                Value::Num(x) => (*x != 0.0 && !x.is_nan()).to_string(),
                _ => "true".into(),
            },
            Ty::Int => js_string(v),
            _ => self.b.lit_value(v),
        }
    }

    // ------------------------------------------------------------ the runtime: the scalar helpers, and each array
    // operation once per shape it is used at
    fn need(&mut self, name: String, body: String) -> String {
        if !self.helpers.iter().any(|(k, _)| *k == name) {
            self.helpers.push((name.clone(), body));
        }
        name
    }
    // t: an array of reals (a vector) or of vectors (a matrix)
    fn vec_op(&mut self, op: &str, t: &Ty) -> String {
        let tt = self.cty(t);
        let Ty::Arr(n, of) = t else { return self.b.fail(format!("no C array operation {op} on a scalar")) };
        let n = *n;
        let mat = matches!(**of, Ty::Arr(..));
        match op {
            "add" | "sub" => {
                let o = if op == "add" { "+" } else { "-" };
                let inner = if mat {
                    let f = self.vec_op(op, of);
                    format!("r.v[i] = {f}(a.v[i], b.v[i]);")
                } else {
                    format!("r.v[i] = a.v[i] {o} b.v[i];")
                };
                let tg = self.tag(t);
                let name = format!("pc_{op}_{tg}");
                let body = format!("static inline {tt} pc_{op}_{tg}({tt} a, {tt} b) {{ {tt} r = a; int i; for (i = 0; i < {n}; i++) {{ {inner} }} return r; }}");
                self.need(name, body)
            }
            "neg" => {
                let inner = if mat {
                    let f = self.vec_op("neg", of);
                    format!("r.v[i] = {f}(a.v[i]);")
                } else {
                    "r.v[i] = -a.v[i];".to_string()
                };
                let tg = self.tag(t);
                let body = format!("static inline {tt} pc_neg_{tg}({tt} a) {{ {tt} r = a; int i; for (i = 0; i < {n}; i++) {{ {inner} }} return r; }}");
                self.need(format!("pc_neg_{tg}"), body)
            }
            "scale" | "scalel" | "div" => {
                let inner = if mat {
                    let f = self.vec_op(op, of);
                    format!("r.v[i] = {f}({});", if op == "scalel" { "s, a.v[i]" } else { "a.v[i], s" })
                } else {
                    match op {
                        "scale" => "r.v[i] = a.v[i] * s;",
                        "scalel" => "r.v[i] = s * a.v[i];",
                        _ => "r.v[i] = a.v[i] / s;",
                    }
                    .to_string()
                };
                let args = if op == "scalel" { format!("double s, {tt} a") } else { format!("{tt} a, double s") };
                let tg = self.tag(t);
                let body = format!("static inline {tt} pc_{op}_{tg}({args}) {{ {tt} r = a; int i; for (i = 0; i < {n}; i++) {{ {inner} }} return r; }}");
                self.need(format!("pc_{op}_{tg}"), body)
            }
            "dot" => {
                let tg = self.tag(t);
                let body = format!("static inline double pc_dot_{tg}({tt} a, {tt} b) {{ double s = a.v[0] * b.v[0]; int i; for (i = 1; i < {n}; i++) {{ s = s + a.v[i] * b.v[i]; }} return s; }}");
                self.need(format!("pc_dot_{tg}"), body)
            }
            "norm" => {
                let tg = self.tag(t);
                let d = self.vec_op("dot", t);
                let body = format!("static inline double pc_norm_{tg}({tt} a) {{ return sqrt({d}(a, a)); }}");
                self.need(format!("pc_norm_{tg}"), body)
            }
            "unit" => {
                let tg = self.tag(t);
                let nm = self.vec_op("norm", t);
                let body = format!("static inline {tt} pc_unit_{tg}({tt} a) {{ double n = pc_fmax({nm}(a), 1e-30); {tt} r = a; int i; for (i = 0; i < {n}; i++) {{ r.v[i] = a.v[i] / n; }} return r; }}");
                self.need(format!("pc_unit_{tg}"), body)
            }
            "cross" => {
                let tg = self.tag(t);
                let body = format!("static inline {tt} pc_cross_{tg}({tt} a, {tt} b) {{ {tt} r; r.v[0] = a.v[1] * b.v[2] - a.v[2] * b.v[1]; r.v[1] = a.v[2] * b.v[0] - a.v[0] * b.v[2]; r.v[2] = a.v[0] * b.v[1] - a.v[1] * b.v[0]; return r; }}");
                self.need(format!("pc_cross_{tg}"), body)
            }
            _ => self.b.fail(format!("no C array operation {op}")),
        }
    }
    fn mv_op(&mut self, mt: &Ty, vt: &Ty) -> String {
        let m = self.cty(mt);
        let vin = self.cty(vt);
        let Ty::Arr(mn, mof) = mt else { return self.b.fail("a matrix product of what is not a matrix") };
        let Ty::Arr(_, el) = &**mof else { return self.b.fail("a matrix product of what is not a matrix") };
        let vout = self.cty(&arr(*mn, (**el).clone()));
        let name = format!("pc_mv_{}", self.tag(mt));
        let d = self.vec_op("dot", mof);
        let body = format!("static inline {vout} {name}({m} a, {vin} v) {{ {vout} r; int i; for (i = 0; i < {mn}; i++) {{ r.v[i] = {d}(a.v[i], v); }} return r; }}");
        self.need(name, body)
    }
    fn mm_op(&mut self, at: &Ty, bt: &Ty, rt: &Ty) -> String {
        let a = self.cty(at);
        let b = self.cty(bt);
        let r = self.cty(rt);
        let (Ty::Arr(an, aof), Ty::Arr(_, bof)) = (at, bt) else { return self.b.fail("a matrix product of what is not a matrix") };
        let (Ty::Arr(k, _), Ty::Arr(bm, _)) = (&**aof, &**bof) else { return self.b.fail("a matrix product of what is not a matrix") };
        let name = format!("pc_mm_{}_{}", self.tag(at), self.tag(bt));
        let body = format!(
            "static inline {r} {name}({a} a, {b} b) {{ {r} r; int i, j, k; for (i = 0; i < {an}; i++) {{ for (j = 0; j < {bm}; j++) {{ double s = a.v[i].v[0] * b.v[0].v[j]; for (k = 1; k < {k}; k++) {{ s = s + a.v[i].v[k] * b.v[k].v[j]; }} r.v[i].v[j] = s; }} }} return r; }}"
        );
        self.need(name, body)
    }
    fn tr_op(&mut self, t: &Ty) -> String {
        let Ty::Arr(n, of) = t else { return self.b.fail("transpose of what is not a matrix") };
        let Ty::Arr(m, el) = &**of else { return self.b.fail("transpose of what is not a matrix") };
        let rt = arr(*m, arr(*n, (**el).clone()));
        let a = self.cty(t);
        let r = self.cty(&rt);
        let name = format!("pc_tr_{}", self.tag(t));
        let body = format!("static inline {r} {name}({a} a) {{ {r} r; int i, j; for (i = 0; i < {n}; i++) {{ for (j = 0; j < {m}; j++) {{ r.v[j].v[i] = a.v[i].v[j]; }} }} return r; }}");
        self.need(name, body)
    }

    // ------------------------------------------------------------ expressions
    fn fname(module: &str, name: &str) -> String {
        format!("{}_{name}", cname(module))
    }
    fn const_name(module: &str, name: &str) -> String {
        format!("{}_{}", cname(module), upper(name))
    }
    /// the type a call of a fn or table returns (its struct when it gives several)
    fn ret_type(&mut self, e: usize) -> String {
        let c = self.b.c;
        let (module, name, outs) = match c.ann[e].target {
            Target::Fn(fi) => (&c.fns[fi].module, &c.fns[fi].name, &c.fns[fi].outs),
            Target::Table(ti) => (&c.tables[ti].module, &c.tables[ti].name, &c.tables[ti].outs),
            _ => return self.b.fail("cannot read properties of undefined (reading 'outs')"),
        };
        if outs.len() == 1 {
            self.cty(&outs[0].ty)
        } else {
            format!("{}_out", Self::fname(module, name))
        }
    }
    fn outs_of(&self, e: usize) -> &'a [crate::ast::Param] {
        let c = self.b.c;
        match c.ann[e].target {
            Target::Fn(fi) => &c.fns[fi].outs,
            Target::Table(ti) => &c.tables[ti].outs,
            _ => &[],
        }
    }

    fn e_want(&mut self, e: usize, want: Option<&Ty>) -> String {
        let et = self.b.ty(e);
        if let (Some(w @ Ty::Arr(_, wof)), Some(et @ Ty::Arr(..))) = (want, et) {
            if leaf_is_real(w) && !leaf_is_real(et) {
                let ExprKind::Arr { items, .. } = &self.b.c.exprs[e].kind else {
                    return self.b.fail("an array of whole numbers where one of reals is wanted: write its elements as reals");
                };
                let wt = self.cty(w);
                let parts: Vec<String> = items.iter().map(|&x| self.e_want(x, Some(wof))).collect();
                return format!("(({wt}){{ {{ {} }} }})", parts.join(", "));
            }
        }
        let s = self.ex(e);
        if is_real(want) && is_int(et) {
            if let ExprKind::Num { v, .. } = &self.b.c.exprs[e].kind {
                return self.b.lit(*v);
            }
            return format!("((double)({s}))");
        }
        s
    }
    fn v_arg(&mut self, x: usize) -> String {
        let t = real_of(self.b.ty_of(x));
        self.e_want(x, Some(&t))
    }
    fn r_arg(&mut self, x: usize) -> String {
        self.e_want(x, Some(&REAL))
    }
    fn idx(&mut self, i: usize) -> String {
        match &self.b.c.exprs[i].kind {
            ExprKind::Num { v, .. } => jsfmt::num(*v),
            _ => format!("({})", self.ex(i)),
        }
    }

    fn ex(&mut self, e: usize) -> String {
        let c = self.b.c;
        let ann = &c.ann[e];
        match &c.exprs[e].kind {
            ExprKind::Num { v, .. } => match &ann.fill {
                Some(f) => self.zero(f),
                None if is_int(ann.ty.as_ref()) => format!("INT64_C({})", jsfmt::num(*v)),
                None => self.b.lit(ann.si),
            },
            ExprKind::Bool(v) => v.to_string(),
            ExprKind::Var(name) => match ann.var {
                VarKind::Pi => "3.141592653589793".into(),
                VarKind::Const(ci) => Self::const_name(&c.consts[ci].module, &c.consts[ci].name),
                VarKind::State => format!("st->{}", cname(name)),
                _ => cname(name),
            },
            ExprKind::Arr { items, .. } => {
                let et = self.b.ty_of(e);
                let tt = self.cty(et);
                if let Some(scale) = ann.scale {
                    let parts: Vec<String> = items
                        .iter()
                        .map(|&x| {
                            let v = match &c.exprs[x].kind {
                                ExprKind::Num { v, .. } => {
                                    if is_int(c.ann[x].ty.as_ref()) {
                                        *v
                                    } else {
                                        c.ann[x].si
                                    }
                                }
                                _ => f64::NAN,
                            };
                            self.b.lit(v * scale)
                        })
                        .collect();
                    return format!("(({tt}){{ {{ {} }} }})", parts.join(", "));
                }
                let el = match et {
                    Ty::Arr(_, of) => Some(&**of),
                    _ => None,
                };
                let parts: Vec<String> = items.iter().map(|&x| self.e_want(x, el)).collect();
                format!("(({tt}){{ {{ {} }} }})", parts.join(", "))
            }
            ExprKind::Index { a, i } => {
                let a = self.ex(*a);
                format!("{a}.v[{}]", self.idx(*i))
            }
            ExprKind::Field { a, f } => format!("{}.{}", self.ex(*a), cname(f)),
            ExprKind::Not(a) => format!("(!{})", self.ex(*a)),
            ExprKind::Neg(a) => match ann.ty.as_ref() {
                Some(t @ Ty::Arr(..)) => {
                    let f = self.vec_op("neg", t);
                    format!("{f}({})", self.ex(*a))
                }
                _ => format!("(-({}))", self.ex(*a)),
            },
            ExprKind::Ifx { c: cc, a, b } => {
                let t = ann.ty.as_ref();
                let cs = self.ex(*cc);
                let as_ = self.e_want(*a, t);
                let bs = self.e_want(*b, t);
                format!("({cs} ? {as_} : {bs})")
            }
            ExprKind::Bin { op, a, b } => self.bin(e, *op, *a, *b),
            ExprKind::Call { args, .. } => self.call(e, args),
        }
    }

    fn bin(&mut self, e: usize, op: BinOp, a: usize, b: usize) -> String {
        let c = self.b.c;
        let ann = &c.ann[e];
        let ety = ann.ty.as_ref();
        match op {
            BinOp::And => return format!("({} && {})", self.ex(a), self.ex(b)),
            BinOp::Or => return format!("({} || {})", self.ex(a), self.ex(b)),
            BinOp::Pow => {
                let k = jsfmt::num(ann.k);
                return if is_int(ety) { format!("pc_ipowi({}, {k})", self.ex(a)) } else { format!("pc_ipow({}, {k})", self.r_arg(a)) };
            }
            _ => {}
        }
        let (at, bt) = (self.b.ty(a), self.b.ty(b));
        if op.is_cmp() {
            let both_int = is_int(at) && is_int(bt);
            let w = if both_int || matches!(at, Some(Ty::Bool)) { None } else { Some(&REAL) };
            let x = self.e_want(a, w);
            let y = self.e_want(b, w);
            return format!("({x} {} {y})", op.text());
        }
        if op == BinOp::Add || op == BinOp::Sub {
            if let Some(et @ Ty::Arr(..)) = ety {
                let f = self.vec_op(if op == BinOp::Add { "add" } else { "sub" }, et);
                let za = if is_arr(at) { self.e_want(a, Some(et)) } else { self.zero(et) };
                let zb = if is_arr(bt) { self.e_want(b, Some(et)) } else { self.zero(et) };
                return format!("{f}({za}, {zb})");
            }
            if is_int(ety) {
                let x = self.ex(a);
                return format!("({x} {} {})", op.text(), self.ex(b));
            }
            let x = self.e_want(a, ety);
            return format!("({x} {} {})", op.text(), self.e_want(b, ety));
        }
        match ann.shape {
            Shape::Mv => {
                let (Some(at), Some(bt)) = (at, bt) else { return self.b.fail("a matrix product with no type") };
                let f = self.mv_op(at, bt);
                let x = self.v_arg(a);
                return format!("{f}({x}, {})", self.v_arg(b));
            }
            Shape::Mm => {
                let (Some(at), Some(bt), Some(et)) = (at, bt, ety) else { return self.b.fail("a matrix product with no type") };
                let f = self.mm_op(at, bt, et);
                let x = self.v_arg(a);
                return format!("{f}({x}, {})", self.v_arg(b));
            }
            Shape::Vs | Shape::Ms => {
                let Some(et) = ety else { return self.b.fail("a scaled array with no type") };
                if op == BinOp::Div {
                    let f = self.vec_op("div", et);
                    let x = self.v_arg(a);
                    return format!("{f}({x}, {})", self.r_arg(b));
                }
                if ann.arr_left {
                    let f = self.vec_op("scale", et);
                    let x = self.v_arg(a);
                    return format!("{f}({x}, {})", self.r_arg(b));
                }
                let f = self.vec_op("scalel", et);
                let x = self.r_arg(a);
                return format!("{f}({x}, {})", self.v_arg(b));
            }
            _ => {}
        }
        if is_int(ety) {
            let x = self.ex(a);
            return format!("({x} {} {})", op.text(), self.ex(b));
        }
        let x = self.r_arg(a);
        format!("({x} {} {})", op.text(), self.r_arg(b))
    }

    fn call(&mut self, e: usize, a: &[usize]) -> String {
        let c = self.b.c;
        let ann = &c.ann[e];
        let ety = ann.ty.as_ref();
        if let Target::Record(ri) = ann.target {
            return self.zero(&Ty::Rec(c.records[ri].name.clone()));
        }
        if let Target::Builtin(f) = ann.target {
            let miss = |g: &mut Self| g.b.fail(format!("{f}: an input is missing"));
            let r = |g: &mut Self, i: usize| match a.get(i) {
                Some(&x) => g.r_arg(x),
                None => miss(g),
            };
            let x = |g: &mut Self, i: usize| match a.get(i) {
                Some(&x) => g.ex(x),
                None => miss(g),
            };
            let v = |g: &mut Self, i: usize| match a.get(i) {
                Some(&x) => g.v_arg(x),
                None => miss(g),
            };
            // the real-element type of the first input (the shape an array operation is made for)
            let real_first = |g: &mut Self| match a.first() {
                Some(&x) => real_of(g.b.ty_of(x)),
                None => {
                    miss(g);
                    REAL
                }
            };
            return match f {
                "sqrt" => format!("sqrt({})", r(self, 0)),
                "abs" => {
                    if is_int(ety) {
                        format!("pc_iabs({})", x(self, 0))
                    } else {
                        format!("pc_fabs({})", r(self, 0))
                    }
                }
                "floor" | "ceil" | "round" => {
                    if is_int(ety) {
                        x(self, 0)
                    } else {
                        format!("{f}({})", r(self, 0))
                    }
                }
                "sign" => format!("pc_sign({})", r(self, 0)),
                "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "exp" | "log" | "log10" => format!("{f}({})", r(self, 0)),
                "atan2" | "fmod" | "pow" => {
                    let p = r(self, 0);
                    format!("{f}({p}, {})", r(self, 1))
                }
                "hypot" => {
                    let p = r(self, 0);
                    format!("pc_hypot({p}, {})", r(self, 1))
                }
                "min" | "max" => {
                    let all_int = a.iter().all(|&x| is_int(c.ann[x].ty.as_ref()));
                    let g = if all_int { format!("pc_i{f}") } else { format!("pc_f{f}") };
                    let args: Vec<String> = a.iter().map(|&x| if all_int { self.ex(x) } else { self.r_arg(x) }).collect();
                    let Some((first, rest)) = args.split_first() else {
                        return self.b.fail("Reduce of empty array with no initial value");
                    };
                    rest.iter().fold(first.clone(), |acc, x| format!("{g}({acc}, {x})"))
                }
                "clamp" => {
                    if is_int(ety) {
                        let (p, q) = (x(self, 0), x(self, 1));
                        format!("pc_imin(pc_imax({p}, {q}), {})", x(self, 2))
                    } else {
                        let (p, q) = (r(self, 0), r(self, 1));
                        format!("pc_clamp({p}, {q}, {})", r(self, 2))
                    }
                }
                "real" => format!("((double)({}))", x(self, 0)),
                "int" => format!("((int64_t)({}))", r(self, 0)),
                "div" | "rem" | "band" | "bor" | "bxor" | "shl" | "shr" => {
                    let o = match f {
                        "div" => "/",
                        "rem" => "%",
                        "band" => "&",
                        "bor" => "|",
                        "bxor" => "^",
                        "shl" => "<<",
                        _ => ">>",
                    };
                    let p = x(self, 0);
                    format!("({p} {o} {})", x(self, 1))
                }
                "len" => match a.first().and_then(|&x| c.ann[x].ty.as_ref()) {
                    Some(Ty::Arr(n, _)) => format!("INT64_C({n})"),
                    _ => "INT64_C(undefined)".into(),
                },
                "dot" | "cross" => {
                    let t = real_first(self);
                    let op = self.vec_op(f, &t);
                    let p = v(self, 0);
                    format!("{op}({p}, {})", v(self, 1))
                }
                "norm" | "unit" => {
                    let t = real_first(self);
                    let op = self.vec_op(f, &t);
                    format!("{op}({})", v(self, 0))
                }
                "transpose" => {
                    let t = real_first(self);
                    let op = self.tr_op(&t);
                    format!("{op}({})", v(self, 0))
                }
                _ => self.b.fail(format!("no C for the builtin {f}")),
            };
        }
        let (module, name, wants, proc_): (&str, &str, Vec<&Ty>, bool) = match ann.target {
            Target::Fn(fi) => {
                let f = &c.fns[fi];
                if a.len() > f.params.len() {
                    return self.b.fail(format!("{}: more inputs than it takes", f.name));
                }
                (&f.module, &f.name, f.params.iter().map(|p| &p.ty).collect(), f.kind == FnKind::Proc)
            }
            Target::Table(ti) => {
                let t = &c.tables[ti];
                (&t.module, &t.name, a.iter().map(|_| &t.key.ty).collect(), false)
            }
            _ => return self.b.fail("a call of nothing"),
        };
        let mut args: Vec<String> = a.iter().zip(wants).map(|(&x, w)| self.e_want(x, Some(w))).collect();
        if proc_ {
            args.insert(0, "st".into()); // never reached: a proc is called by the harness, not from code
        }
        format!("{}({})", Self::fname(module, name), args.join(", "))
    }

    fn lv(&mut self, l: usize) -> String {
        let c = self.b.c;
        match &c.exprs[l].kind {
            ExprKind::Var(name) => {
                if c.ann[l].var == VarKind::State {
                    format!("st->{}", cname(name))
                } else {
                    cname(name)
                }
            }
            ExprKind::Index { a, i } => {
                let base = self.lv(*a);
                format!("{base}.v[{}]", self.idx(*i))
            }
            ExprKind::Field { a, f } => format!("{}.{}", self.lv(*a), cname(f)),
            _ => self.b.fail("not a place to set"),
        }
    }

    // ------------------------------------------------------------ statements
    fn stmts(&mut self, body: &[Stmt], ind: usize) -> String {
        body.iter().map(|s| self.stmt(s, ind)).collect()
    }
    fn stmt(&mut self, s: &Stmt, ind: usize) -> String {
        let c = self.b.c;
        let ii = "    ".repeat(ind);
        match &s.kind {
            StmtKind::Let { names, e, .. } => {
                if names.len() > 1 {
                    let outs = self.outs_of(*e);
                    let n = format!("t__{}", self.tmp);
                    self.tmp += 1;
                    let rt = self.ret_type(*e);
                    let mut out = format!("{ii}{rt} {n} = {};\n", self.ex(*e));
                    for (i, nm) in names.iter().enumerate() {
                        let Some(o) = outs.get(i) else { return self.b.fail("cannot read properties of undefined (reading 'ty')") };
                        out += &format!("{ii}{} {} = {n}.{};\n", self.cty(&o.ty), cname(nm), cname(&o.name));
                    }
                    return out;
                }
                let Some(vty) = c.ann[*e].vty.as_ref() else { return self.b.fail("a let with no type") };
                let t = self.cty(vty);
                format!("{ii}{t} {} = {};\n", cname(&names[0]), self.e_want(*e, Some(vty)))
            }
            StmtKind::State { .. } => String::new(),
            StmtKind::Set { targets, e } => {
                if targets.len() > 1 {
                    let outs = self.outs_of(*e);
                    let n = format!("t__{}", self.tmp);
                    self.tmp += 1;
                    let rt = self.ret_type(*e);
                    let mut out = format!("{ii}{{\n{ii}    {rt} {n} = {};\n", self.ex(*e));
                    for (i, t) in targets.iter().enumerate() {
                        let l = self.lv(*t);
                        let Some(o) = outs.get(i) else { return self.b.fail("cannot read properties of undefined (reading 'name')") };
                        out += &format!("{ii}    {l} = {n}.{};\n", cname(&o.name));
                    }
                    return out + &format!("{ii}}}\n");
                }
                let l = self.lv(targets[0]);
                let tt = c.ann[targets[0]].ty.as_ref();
                format!("{ii}{l} = {};\n", self.e_want(*e, tt))
            }
            StmtKind::If { arms, els } => {
                let mut out = String::new();
                for (i, (cond, body)) in arms.iter().enumerate() {
                    let lead = if i > 0 { " else if".to_string() } else { format!("{ii}if") };
                    let cs = self.ex(*cond);
                    out += &format!("{lead} ({cs}) {{\n{}{ii}}}", self.stmts(body, ind + 1));
                }
                if let Some(els) = els {
                    out += &format!(" else {{\n{}{ii}}}", self.stmts(els, ind + 1));
                }
                out + "\n"
            }
            StmtKind::For { v, a, b, body } => {
                let n = format!("end__{}", self.tmp);
                self.tmp += 1;
                let v = cname(v);
                let bs = self.ex(*b);
                let as_ = self.ex(*a);
                let inner = self.stmts(body, ind + 2);
                format!("{ii}{{\n{ii}    int64_t {n} = {bs};\n{ii}    int64_t {v} = {as_};\n{ii}    while ({v} < {n}) {{\n{inner}{ii}        {v} += 1;\n{ii}    }}\n{ii}}}\n")
            }
            StmtKind::Settle { n: ne, c: ce, body, els } => {
                let k = format!("k__{}", self.tmp);
                self.tmp += 1;
                let n = format!("n__{}", self.tmp);
                self.tmp += 1;
                let ns = self.ex(*ne);
                let inner = self.stmts(body, ind + 2);
                let cs = self.ex(*ce);
                let es = match els {
                    Some(els) => self.stmts(els, ind + 3),
                    None => String::new(),
                };
                format!(
                    "{ii}{{\n{ii}    int64_t {n} = {ns};\n{ii}    int64_t {k} = 0;\n{ii}    for (;;) {{\n{inner}{ii}        {k} += 1;\n{ii}        if ({cs}) {{ break; }}\n{ii}        if ({k} >= {n}) {{\n{es}{ii}            break;\n{ii}        }}\n{ii}    }}\n{ii}}}\n"
                )
            }
        }
    }

    fn state_type(&mut self, module: &str, name: &str) -> String {
        format!("{}_{}State", cname(module), self.b.pascal(name))
    }

    // ------------------------------------------------------------ the dispatcher: a function by name, flattened
    // statements filling `into` from x[at..]
    fn read_arg(&self, t: &Ty, at: usize, into: &str, ii: &str) -> (String, usize) {
        match t {
            Ty::Real(_) => (format!("{ii}{into} = x[{at}];\n"), 1),
            Ty::Int => (format!("{ii}{into} = (int64_t)x[{at}];\n"), 1),
            Ty::Bool => (format!("{ii}{into} = x[{at}] != 0.0;\n"), 1),
            Ty::Rec(name) => {
                let (mut s, mut k) = (String::new(), 0);
                if let Some(r) = self.b.record(name) {
                    for f in &r.fields {
                        let (cs, n) = self.read_arg(&f.ty, at + k, &format!("{into}.{}", cname(&f.name)), ii);
                        s += &cs;
                        k += n;
                    }
                }
                (s, k)
            }
            Ty::Arr(n, of) => {
                let (mut s, mut k) = (String::new(), 0);
                for i in 0..*n {
                    let (cs, m) = self.read_arg(of, at + k, &format!("{into}.v[{i}]"), ii);
                    s += &cs;
                    k += m;
                }
                (s, k)
            }
            Ty::Tuple(_) => (String::new(), 0),
        }
    }
    fn push_out(&self, t: &Ty, v: &str, ii: &str) -> String {
        match t {
            Ty::Real(_) => format!("{ii}out[(*ny)++] = {v};\n"),
            Ty::Int => format!("{ii}out[(*ny)++] = (double){v};\n"),
            Ty::Bool => format!("{ii}out[(*ny)++] = {v} ? 1.0 : 0.0;\n"),
            Ty::Rec(name) => match self.b.record(name) {
                Some(r) => r.fields.iter().map(|f| self.push_out(&f.ty, &format!("{v}.{}", cname(&f.name)), ii)).collect(),
                None => String::new(),
            },
            Ty::Arr(n, of) => (0..*n).map(|i| self.push_out(of, &format!("{v}.v[{i}]"), ii)).collect(),
            Ty::Tuple(_) => String::new(),
        }
    }
    fn call_body(&mut self, f: &crate::ast::Func, ii: &str) -> (String, usize) {
        let mut s = String::new();
        for (i, p) in f.params.iter().enumerate() {
            s += &format!("{ii}{} a{i};\n", self.cty(&p.ty));
        }
        let mut at = 0;
        for (i, p) in f.params.iter().enumerate() {
            let (cs, n) = self.read_arg(&p.ty, at, &format!("a{i}"), ii);
            s += &cs;
            at += n;
        }
        (s, at)
    }
    fn outs_body(&mut self, f: &crate::ast::Func, call: &str, ii: &str) -> String {
        let inner = format!("{ii}    ");
        if f.outs.len() == 1 {
            let t = self.cty(&f.outs[0].ty);
            return format!("{ii}{{\n{ii}    {t} r = {call};\n{}{ii}}}\n", self.push_out(&f.outs[0].ty, "r", &inner));
        }
        let pushes: String = f.outs.iter().map(|o| self.push_out(&o.ty, &format!("r.{}", cname(&o.name)), &inner)).collect();
        format!("{ii}{{\n{ii}    {}_out r = {call};\n{pushes}{ii}}}\n", Self::fname(&f.module, &f.name))
    }

    fn run(&mut self, title: Option<&str>, lib: &str) -> Files {
        let c = self.b.c;
        let head = head();
        let mut files = Files::new();
        // ------------------------------------------------------------ each module: its constants, tables and functions
        let mut decls: Vec<String> = Vec::new(); // the header's declarations, in program order
        for (mi, (mname, mdoc)) in c.modules.iter().enumerate() {
            let what = no_close(&mdoc.join(" "));
            let what = if what.is_empty() { format!("the pseudocode module {mname}") } else { what };
            let mut out = format!("/* {mname}: {what} */\n/* {head} */\n#include \"{lib}.h\"\n#include \"{lib}_rt.h\"\n\n");
            for item in &c.mod_items[mi] {
                match *item {
                    ItemRef::Const(ci) => {
                        let k = &c.consts[ci];
                        let v = self.b.const_of(k.e);
                        let t = self.b.ty_of(k.e);
                        let name = Self::const_name(mname, &k.name);
                        let ct = self.cty(t);
                        decls.push(format!("{}extern const {ct} {name};\n", doc(&k.doc)));
                        let ct = self.cty(t);
                        let iv = self.init(&v, t);
                        out += &format!("const {ct} {name} = {iv};\n\n");
                    }
                    ItemRef::Record(ri) => self.record_type(&c.records[ri].name),
                    ItemRef::Table(ti) => {
                        let t = &c.tables[ti];
                        let (cn, rn) = (t.outs.len(), t.si.len());
                        let f = Self::fname(mname, &t.name);
                        let tn = format!("{f}_rows");
                        let rows: String = t
                            .si
                            .iter()
                            .map(|r| {
                                let xs: Vec<String> = r.iter().map(|x| self.b.lit(*x)).collect();
                                format!("    {{ {} }},\n", xs.join(", "))
                            })
                            .collect();
                        out += &format!("static const double {tn}[{rn}][{cn}] = {{\n{rows}}};\n");
                        if cn > 1 {
                            let fs: Vec<String> = t.outs.iter().map(|o| format!("double {};", cname(&o.name))).collect();
                            decls.push(format!("typedef struct {{ {} }} {f}_out;\n", fs.join(" ")));
                        }
                        let ret = if cn == 1 { "double".to_string() } else { format!("{f}_out") };
                        let key = cname(&t.key.name);
                        let mode = mode_text(t.mode);
                        let outs: Vec<&str> = t.outs.iter().map(|o| o.name.as_str()).collect();
                        decls.push(format!("{}/* Table ({mode}): {} -> {}. */\n{ret} {f}(double {key});\n", doc(&t.doc), t.key.name, outs.join(", ")));
                        let give = if cn == 1 {
                            "    return r[0];\n".to_string()
                        } else {
                            let sets: String = t.outs.iter().enumerate().map(|(i, o)| format!("        o.{} = r[{i}];\n", cname(&o.name))).collect();
                            format!("    {{\n        {f}_out o;\n{sets}        return o;\n    }}\n")
                        };
                        out += &format!("{ret} {f}(double {key}) {{\n    double r[{cn}];\n    pc_lookup_{mode}(&{tn}[0][0], {rn}, {cn}, {key}, r);\n{give}}}\n\n");
                    }
                    ItemRef::Fn(fi) => {
                        let f = &c.fns[fi];
                        let fnm = Self::fname(mname, &f.name);
                        let mut ins: Vec<String> = f.params.iter().map(|p| format!("{} {}", self.cty(&p.ty), cname(&p.name))).collect();
                        if f.kind == FnKind::Proc {
                            let s = self.state_type(mname, &f.name);
                            let fields: String = f
                                .states
                                .iter()
                                .map(|(name, e)| {
                                    let t = c.ann[*e].vty.as_ref().unwrap_or(&REAL);
                                    format!("    {} {};\n", self.cty(t), cname(name))
                                })
                                .collect();
                            decls.push(format!("/* The state {} keeps between calls. */\ntypedef struct {{\n{fields}}} {s};\n", f.name));
                            decls.push(format!("/* {}'s state before its first call. */\nvoid {fnm}_init({s} *st);\n", f.name));
                            let starts: String = f
                                .states
                                .iter()
                                .map(|(name, e)| {
                                    let t = c.ann[*e].vty.as_ref().unwrap_or(&REAL);
                                    let tmp_name = format!("s__{}", cname(name));
                                    let ct = self.cty(t);
                                    let v = self.b.const_of(*e);
                                    let iv = self.init(&v, t);
                                    format!("    {{\n        static const {ct} {tmp_name} = {iv};\n        st->{} = {tmp_name};\n    }}\n", cname(name))
                                })
                                .collect();
                            out += &format!("void {fnm}_init({s} *st) {{\n{starts}}}\n\n");
                            ins.insert(0, format!("{s} *st"));
                        }
                        if f.outs.len() > 1 {
                            let fs: Vec<String> = f.outs.iter().map(|o| format!("{} {};", self.cty(&o.ty), cname(&o.name))).collect();
                            decls.push(format!("typedef struct {{ {} }} {fnm}_out;\n", fs.join(" ")));
                        }
                        let ret = if f.outs.len() == 1 { self.cty(&f.outs[0].ty) } else { format!("{fnm}_out") };
                        let sig = format!("{ret} {fnm}({})", if ins.is_empty() { "void".to_string() } else { ins.join(", ") });
                        let pdoc = doc(&f.params.iter().map(|p| format!("- {}", p.name)).collect::<Vec<_>>());
                        let odoc = doc(&f.outs.iter().map(|o| format!("- returns {}", o.name)).collect::<Vec<_>>());
                        decls.push(format!("{}{pdoc}{odoc}{sig};\n", doc(&f.doc)));
                        let lets: String = f
                            .outs
                            .iter()
                            .map(|o| {
                                let t = self.cty(&o.ty);
                                format!("    {t} {} = {};\n", cname(&o.name), self.zero(&o.ty))
                            })
                            .collect();
                        out += &format!("{sig} {{\n{lets}");
                        out += &self.stmts(&f.body, 1);
                        if f.outs.len() == 1 {
                            out += &format!("    return {};\n", cname(&f.outs[0].name));
                        } else {
                            let sets: String = f.outs.iter().map(|o| format!("        r__.{0} = {0};\n", cname(&o.name))).collect();
                            out += &format!("    {{\n        {ret} r__;\n{sets}        return r__;\n    }}\n");
                        }
                        out += "}\n\n";
                    }
                }
            }
            set_file(&mut files, format!("src/{mname}.c"), format!("{}\n", trim_end_js(&out)));
        }

        let mut disp = format!("/* The vector dispatcher: a function by its name (module::name), its inputs and outputs flattened (SI). */\n/* {head} */\n#include <string.h>\n#include \"{lib}.h\"\n\n");
        disp += "/* 0 and the outputs in out (*ny of them), or -1: no such function, or not the number of inputs it takes. */\nint pc_call(const char *name, const double *x, int nx, double *out, int *ny) {\n    *ny = 0;\n";
        for f in c.fns.iter().filter(|g| g.kind == FnKind::Fn) {
            let (setup, at) = self.call_body(f, "        ");
            let call = format!("{}({})", Self::fname(&f.module, &f.name), (0..f.params.len()).map(|i| format!("a{i}")).collect::<Vec<_>>().join(", "));
            let ob = self.outs_body(f, &call, "        ");
            disp += &format!(
                "    if (strcmp(name, \"{}::{}\") == 0) {{\n        if (nx != {at}) {{ return -1; }}\n    {{\n{setup}{ob}    }}\n        return 0;\n    }}\n",
                f.module, f.name
            );
        }
        disp += "    (void)x; (void)nx; (void)out;\n    return -1;\n}\n\n";
        disp += "/* A proc called once per call, its state carried: x holds ncalls rows of nx inputs; out gets ncalls rows of\n   *ny outputs each. 0, or -1. */\nint pc_call_seq(const char *name, const double *xs, int ncalls, int nx, double *outs, int *ny) {\n    int c;\n    *ny = 0;\n";
        for f in c.fns.iter().filter(|g| g.kind == FnKind::Proc) {
            let (setup, at) = self.call_body(f, "            ");
            let st = self.state_type(&f.module, &f.name);
            let fnm = Self::fname(&f.module, &f.name);
            let args: Vec<String> = (0..f.params.len()).map(|i| format!("a{i}")).collect();
            let call = format!("{fnm}(&st{}{})", if f.params.is_empty() { "" } else { ", " }, args.join(", "));
            let ob = self.outs_body(f, &call, "            ");
            disp += &format!(
                "    if (strcmp(name, \"{}::{}\") == 0) {{\n        {st} st;\n        if (nx != {at}) {{ return -1; }}\n        {fnm}_init(&st);\n        for (c = 0; c < ncalls; c++) {{\n            const double *x = xs + (long)c * nx;\n            double *out = outs + (long)c * 256;\n            *ny = 0;\n            {{\n{setup}{ob}            }}\n            (void)x;\n        }}\n        return 0;\n    }}\n",
                f.module, f.name
            );
        }
        disp += "    (void)xs; (void)ncalls; (void)nx; (void)outs; (void)c;\n    return -1;\n}\n";
        set_file(&mut files, "src/dispatch.c".into(), disp);

        // ------------------------------------------------------------ the header and the runtime
        let guard = format!("{}_H", guard_of(lib));
        let title = title.filter(|t| !t.is_empty()).unwrap_or("Functions written in the pseudocode");
        let types: Vec<String> = self.types.iter().map(|(_, d)| d.clone().unwrap_or_else(|| "null".into())).collect();
        set_file(
            &mut files,
            format!("include/{lib}.h"),
            format!(
                "/* {title}. {head} */\n/* Every relation is SI in and SI out. C99, no dynamic memory. */\n#ifndef {guard}\n#define {guard}\n#include <stdbool.h>\n#include <stdint.h>\n\n{}\n\n{}\nint pc_call(const char *name, const double *x, int nx, double *out, int *ny);\nint pc_call_seq(const char *name, const double *xs, int ncalls, int nx, double *outs, int *ny);\n\n#endif\n",
                types.join("\n"),
                decls.join("\n")
            ),
        );
        let rguard = format!("{}_RT_H", guard_of(lib));
        let helpers: Vec<&str> = self.helpers.iter().map(|(_, b)| b.as_str()).collect();
        set_file(
            &mut files,
            format!("include/{lib}_rt.h"),
            format!(
                "/* The arithmetic every translation shares with the interpreter (design/js/pcode.js `rt`). {head} */\n#ifndef {rguard}\n#define {rguard}\n#include <math.h>\n#include <stdint.h>\n#include \"{lib}.h\"\n\n{C_RT}\n{}\n\n#endif\n",
                helpers.join("\n")
            ),
        );
        files
    }
}

/// The scalar runtime every C translation carries (pcode_c.js `C_RT`).
const C_RT: &str = r#"/* x^k by repeated multiplication, left to right; x^-k = 1/(x^k). */
static inline double pc_ipow(double x, int k) { double r = x; int i, n = k < 0 ? -k : k; if (k == 0) { return 1.0; } for (i = 1; i < n; i++) { r = r * x; } return k < 0 ? 1.0 / r : r; }
static inline int64_t pc_ipowi(int64_t x, int k) { int64_t r = 1; int i; for (i = 0; i < k; i++) { r *= x; } return r; }
static inline double pc_fmin(double a, double b) { return b < a ? b : a; }
static inline double pc_fmax(double a, double b) { return b > a ? b : a; }
static inline int64_t pc_imin(int64_t a, int64_t b) { return b < a ? b : a; }
static inline int64_t pc_imax(int64_t a, int64_t b) { return b > a ? b : a; }
static inline int64_t pc_iabs(int64_t a) { return a < 0 ? -a : a; }
static inline double pc_fabs(double x) { if (x < 0.0) { return -x; } if (x == 0.0) { return 0.0; } return x; }
static inline double pc_clamp(double x, double lo, double hi) { return pc_fmin(pc_fmax(x, lo), hi); }
static inline double pc_sign(double x) { return x > 0.0 ? 1.0 : (x < 0.0 ? -1.0 : 0.0); }
static inline double pc_hypot(double a, double b) { return sqrt(a * a + b * b); }
/* The last row whose key is at or below x (the first row below the first key). */
static inline void pc_lookup_step(const double *t, int rows, int cols, double x, double *r) {
    int i = 0, j;
    while (i + 1 < rows && t[(i + 1) * cols] <= x) { i++; }
    for (j = 0; j < cols; j++) { r[j] = t[i * cols + j]; }
}
/* Straight-line between the two rows around x, held at the ends. */
static inline void pc_lookup_linear(const double *t, int rows, int cols, double x, double *r) {
    double xc = pc_clamp(x, t[0], t[(rows - 1) * cols]), s;
    int i = 0, j;
    while (i + 2 < rows && t[(i + 1) * cols] <= xc) { i++; }
    s = (xc - t[i * cols]) / (t[(i + 1) * cols] - t[i * cols]);
    for (j = 0; j < cols; j++) { r[j] = t[i * cols + j] + s * (t[(i + 1) * cols + j] - t[i * cols + j]); }
}"#;
