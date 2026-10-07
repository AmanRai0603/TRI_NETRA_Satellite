//! The translator to Rust: design/js/pcode_gen.js `toRust`, line for line. A module is a Rust
//! module, a record a struct, a table a function over a constant array, a proc a function handed
//! its state; the arithmetic is the interpreter's (`rt`), every literal already in SI.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

use super::*;
use crate::ast::{BinOp, ExprKind, FnKind, Stmt, StmtKind, TableMode};
use crate::check::{ItemRef, Shape, Target, VarKind};

pub(super) fn to_rust(i: &Interp, o: &RustOptions) -> Result<Files, String> {
    let mut g = Gen {
        b: Base::new(i, ["f64::INFINITY", "f64::NEG_INFINITY", "f64::NAN"]),
        tmp: 0,
        sort: false,
        stream: std::cell::Cell::new(false),
        erf: false,
        root: o.root.filter(|r| !r.is_empty()).unwrap_or("crate").to_string(),
        math: o.math.filter(|m| !m.is_empty()).map(str::to_string),
        dispatch: o.dispatch,
    };
    let files = g.run(o.title);
    g.b.done(files)
}

struct Gen<'a> {
    b: Base<'a>,
    tmp: usize,
    /// runtime pieces the program uses beyond RUST_RT (rt.rs carries them only then)
    sort: bool,
    stream: std::cell::Cell<bool>,
    erf: bool,
    /// the path the modules are under (`crate`, or `crate::alg` when embedded in a crate)
    root: String,
    /// the module the scalar maths comes from (a no_std crate's), else `f64`'s own
    math: Option<String>,
    /// whether to write the vector dispatcher (a test aid that needs the heap)
    dispatch: bool,
}

fn mode_text(m: TableMode) -> &'static str {
    match m {
        TableMode::Step => "step",
        TableMode::Linear => "linear",
    }
}

impl<'a> Gen<'a> {
    /// a scalar maths function: `f64`'s own (`std`), or the named one of the maths module
    fn fm(&self, std: &str, name: &str) -> String {
        match &self.math {
            Some(m) => format!("{m}::{name}"),
            None => std.to_string(),
        }
    }
    fn rty(&mut self, t: &Ty) -> String {
        match t {
            Ty::Real(_) => "f64".into(),
            Ty::Int | Ty::Choice(_) => "i64".into(),
            Ty::Bool => "bool".into(),
            Ty::Arr(n, of) => format!("[{}; {n}]", self.rty(of)),
            Ty::Rec(name) => name.clone(),
            Ty::Stream => {
                self.stream.set(true);
                "rt::Stream".into()
            }
            Ty::Tuple(_) | Ty::Str => self.b.fail(format!("no Rust type for {}", t.text())),
        }
    }
    fn zero(&self, t: &Ty) -> String {
        match t {
            Ty::Real(_) => "0.0".into(),
            Ty::Int | Ty::Choice(_) => "0".into(),
            Ty::Bool => "false".into(),
            Ty::Arr(n, of) => format!("[{}; {n}]", self.zero(of)),
            Ty::Rec(name) => format!("{name}::default()"),
            Ty::Stream => {
                self.stream.set(true);
                "rt::Stream::default()".into()
            }
            Ty::Tuple(_) | Ty::Str => "undefined".into(),
        }
    }
    fn value_lit(&mut self, v: &Value, t: &Ty) -> String {
        match t {
            Ty::Arr(_, of) => match v {
                Value::Arr(items) => {
                    let parts: Vec<String> = items.iter().map(|x| self.value_lit(x, of)).collect();
                    format!("[{}]", parts.join(", "))
                }
                _ => self.b.fail("v.map is not a function"),
            },
            Ty::Int | Ty::Bool | Ty::Choice(_) => js_string(v),
            Ty::Stream => {
                self.stream.set(true);
                let Value::Arr(items) = v else { return self.b.fail("v.map is not a function") };
                let ws: Vec<String> = items.iter().map(|x| self.b.lit_value(x)).collect();
                format!("rt::Stream::from_words([{}])", ws.join(", "))
            }
            _ => self.b.lit_value(v),
        }
    }

    // an expression, made to fit the type it is wanted as (int -> real)
    fn e_want(&mut self, e: usize, want: Option<&Ty>) -> String {
        let et = self.b.ty(e);
        if let (Some(w @ Ty::Arr(_, wof)), Some(et @ Ty::Arr(..))) = (want, et) {
            if leaf_is_real(w) && !leaf_is_real(et) {
                let ExprKind::Arr { items, .. } = &self.b.c.exprs[e].kind else {
                    return self.b.fail("an array of whole numbers where one of reals is wanted: write its elements as reals");
                };
                let parts: Vec<String> = items.iter().map(|&x| self.e_want(x, Some(wof))).collect();
                return format!("[{}]", parts.join(", "));
            }
        }
        let s = self.ex(e);
        if is_real(want) && is_int(et) {
            if let ExprKind::Num { v, .. } = &self.b.c.exprs[e].kind {
                return self.b.lit(*v);
            }
            return format!("({s} as f64)");
        }
        s
    }
    // an argument of a vector builtin: real elements, whatever the literal was written with
    fn v_arg(&mut self, x: usize) -> String {
        let t = real_of(self.b.ty_of(x));
        self.e_want(x, Some(&t))
    }
    fn idx(&mut self, i: usize) -> String {
        match &self.b.c.exprs[i].kind {
            ExprKind::Num { v, .. } => jsfmt::num(*v),
            _ => format!("({}) as usize", self.ex(i)),
        }
    }

    fn ex(&mut self, e: usize) -> String {
        let c = self.b.c;
        let ann = &c.ann[e];
        match &c.exprs[e].kind {
            ExprKind::Num { v, .. } => match &ann.fill {
                Some(f) => self.zero(f),
                None if is_int(ann.ty.as_ref()) => jsfmt::num(*v),
                None => self.b.lit(ann.si),
            },
            ExprKind::Bool(v) => v.to_string(),
            ExprKind::Str(s) => jsfmt::json_str(s),
            ExprKind::Var(name) => match ann.var {
                VarKind::Pi => "core::f64::consts::PI".into(),
                VarKind::Inf | VarKind::Nan => self.b.non_finite(const_value(ann.var)),
                VarKind::Const(ci) => format!("{}::{}::{}", self.root, c.consts[ci].module, upper(name)),
                VarKind::Data(di) => format!("{}::{}::DATA_{}", self.root, c.data[di].module, upper(name)),
                VarKind::State => format!("st.{name}"),
                VarKind::InOut => format!("(*{name})"),
                _ => name.clone(),
            },
            ExprKind::Arr { items, .. } => {
                let el = match self.b.ty_of(e) {
                    Ty::Arr(_, of) => Some(&**of),
                    _ => None,
                };
                if let Some(scale) = ann.scale {
                    let parts: Vec<String> = items.iter().map(|&x| self.scaled(x, scale)).collect();
                    return format!("[{}]", parts.join(", "));
                }
                let parts: Vec<String> = items.iter().map(|&x| self.e_want(x, el)).collect();
                format!("[{}]", parts.join(", "))
            }
            ExprKind::Index { a, i } => {
                let a = self.ex(*a);
                format!("{a}[{}]", self.idx(*i))
            }
            ExprKind::Field { a, f } => match ann.choice {
                Some((ci, _)) => format!("{}::{}::{}_{}", self.root, c.choices[ci].module, upper(&c.choices[ci].name), upper(f)),
                None => format!("{}.{f}", self.ex(*a)),
            },
            ExprKind::Not(a) => format!("(!{})", self.ex(*a)),
            ExprKind::Neg(a) => match ann.ty.as_ref() {
                Some(Ty::Arr(_, of)) => {
                    let m = if matches!(**of, Ty::Arr(..)) { "rt::mneg" } else { "rt::vneg" };
                    format!("{m}({})", self.ex(*a))
                }
                _ => format!("(-({}))", self.ex(*a)),
            },
            ExprKind::Ifx { c: cc, a, b } => {
                let t = ann.ty.as_ref();
                let cs = self.ex(*cc);
                let as_ = self.e_want(*a, t);
                let bs = self.e_want(*b, t);
                format!("(if {cs} {{ {as_} }} else {{ {bs} }})")
            }
            ExprKind::Bin { op, a, b } => self.bin(e, *op, *a, *b),
            ExprKind::Call { args, .. } => self.call(e, args),
        }
    }
    /// an item of an array written with a unit, scaled to SI (`x.v` or `x.si` times the scale)
    fn scaled(&mut self, x: usize, scale: f64) -> String {
        let c = self.b.c;
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
                return if is_int(ety) { format!("rt::ipowi({}, {k})", self.ex(a)) } else { format!("rt::ipow({}, {k})", self.e_want(a, Some(&REAL))) };
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
            if let Some(et @ Ty::Arr(_, of)) = ety {
                let m = if matches!(**of, Ty::Arr(..)) { "m" } else { "v" };
                let f = if op == BinOp::Add { "add" } else { "sub" };
                let za = if is_arr(at) { self.ex(a) } else { self.zero(et) };
                let zb = if is_arr(bt) { self.ex(b) } else { self.zero(et) };
                return format!("rt::{m}{f}({za}, {zb})");
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
                let x = self.ex(a);
                return format!("rt::mv({x}, {})", self.ex(b));
            }
            Shape::Mm => {
                let x = self.ex(a);
                return format!("rt::mm({x}, {})", self.ex(b));
            }
            Shape::Vs | Shape::Ms => {
                let m = if ann.shape == Shape::Ms { "m" } else { "v" };
                if op == BinOp::Div {
                    let x = self.ex(a);
                    return format!("rt::{m}div({x}, {})", self.e_want(b, Some(&REAL)));
                }
                if ann.arr_left {
                    let x = self.ex(a);
                    return format!("rt::{m}scale({x}, {})", self.e_want(b, Some(&REAL)));
                }
                let x = self.e_want(a, Some(&REAL));
                return format!("rt::{m}scale_l({x}, {})", self.ex(b));
            }
            _ => {}
        }
        if is_int(ety) {
            let x = self.ex(a);
            return format!("({x} {} {})", op.text(), self.ex(b));
        }
        let x = self.e_want(a, Some(&REAL));
        format!("({x} {} {})", op.text(), self.e_want(b, Some(&REAL)))
    }

    fn call(&mut self, e: usize, a: &[usize]) -> String {
        let c = self.b.c;
        let ann = &c.ann[e];
        let ety = ann.ty.as_ref();
        if let Target::Record(ri) = ann.target {
            return format!("{}::default()", c.records[ri].name);
        }
        if let Target::Builtin(f) = ann.target {
            let r = |g: &mut Self, i: usize| match a.get(i) {
                Some(&x) => g.e_want(x, Some(&REAL)),
                None => g.b.fail(format!("{f}: an input is missing")),
            };
            let x = |g: &mut Self, i: usize| match a.get(i) {
                Some(&x) => g.ex(x),
                None => g.b.fail(format!("{f}: an input is missing")),
            };
            let v = |g: &mut Self, i: usize| match a.get(i) {
                Some(&x) => g.v_arg(x),
                None => g.b.fail(format!("{f}: an input is missing")),
            };
            match f {
                "sqrt" => return format!("{}({})", self.fm("f64::sqrt", "sqrt"), r(self, 0)),
                "abs" => return if is_int(ety) { format!("({}).abs()", x(self, 0)) } else { format!("rt::fabs({})", r(self, 0)) },
                "floor" | "ceil" | "round" | "trunc" => return if is_int(ety) { x(self, 0) } else { format!("{}({})", self.fm(&format!("f64::{f}"), f), r(self, 0)) },
                "sign" => return format!("rt::sign({})", r(self, 0)),
                "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "exp" | "log10" => return format!("{}({})", self.fm(&format!("f64::{f}"), f), r(self, 0)),
                "log" => return format!("{}({})", self.fm("f64::ln", "log"), r(self, 0)),
                "log2" => return format!("{}({})", self.fm("f64::log2", "log2"), r(self, 0)),
                // std has no erf: the platform's C library's, declared in rt.rs, the same erf C and MATLAB call
                "erf" => {
                    let p = r(self, 0);
                    if let Some(m) = &self.math {
                        return format!("{m}::erf({p})");
                    }
                    self.erf = true;
                    return format!("rt::erf({p})");
                }
                "atan2" => {
                    let p = r(self, 0);
                    return format!("{}({p}, {})", self.fm("f64::atan2", "atan2"), r(self, 1));
                }
                "hypot" => {
                    let p = r(self, 0);
                    return format!("{}({p}, {})", self.fm("f64::hypot", "hypot"), r(self, 1));
                }
                "fmod" => {
                    let p = r(self, 0);
                    let q = r(self, 1);
                    return match &self.math {
                        Some(m) => format!("{m}::fmod({p}, {q})"),
                        None => format!("({p} % {q})"),
                    };
                }
                "pow" => {
                    // std's powf is an LLVM intrinsic the optimiser rewrites (pow(x, 2.0) to x*x): the exponent goes
                    // through black_box so the platform's pow is called, as the language says, in every build
                    let p = r(self, 0);
                    let q = r(self, 1);
                    return match &self.math {
                        Some(m) => format!("{m}::pow({p}, {q})"),
                        None => format!("f64::powf({p}, core::hint::black_box({q}))"),
                    };
                }
                "min" | "max" => {
                    let all_int = a.iter().all(|&x| is_int(c.ann[x].ty.as_ref()));
                    let g = if all_int { format!("rt::i{f}") } else { format!("rt::f{f}") };
                    let args: Vec<String> = a.iter().map(|&x| if all_int { self.ex(x) } else { self.e_want(x, Some(&REAL)) }).collect();
                    let Some((first, rest)) = args.split_first() else {
                        return self.b.fail("Reduce of empty array with no initial value");
                    };
                    return rest.iter().fold(first.clone(), |acc, x| format!("{g}({acc}, {x})"));
                }
                "clamp" => {
                    if is_int(ety) {
                        let (p, q) = (x(self, 0), x(self, 1));
                        return format!("rt::imin(rt::imax({p}, {q}), {})", x(self, 2));
                    }
                    let (p, q) = (r(self, 0), r(self, 1));
                    return format!("rt::clamp({p}, {q}, {})", r(self, 2));
                }
                "real" => return format!("({} as f64)", x(self, 0)),
                "isnan" => return format!("({}).is_nan()", r(self, 0)),
                "isfinite" => return format!("({}).is_finite()", r(self, 0)),
                "int" => return format!("({} as i64)", r(self, 0)),
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
                    return format!("({p} {o} {})", x(self, 1));
                }
                "len" => {
                    return match a.first().and_then(|&x| c.ann[x].ty.as_ref()) {
                        Some(Ty::Arr(n, _)) => n.to_string(),
                        _ => "undefined".into(),
                    }
                }
                "dot" => {
                    let p = v(self, 0);
                    return format!("rt::dot({p}, {})", v(self, 1));
                }
                "cross" => {
                    let p = v(self, 0);
                    return format!("rt::cross({p}, {})", v(self, 1));
                }
                "norm" => return format!("rt::norm({})", v(self, 0)),
                "unit" => return format!("rt::unit({})", v(self, 0)),
                "transpose" => return format!("rt::tr({})", v(self, 0)),
                "sort" | "argsort" => {
                    self.sort = true;
                    return format!("rt::{f}({})", x(self, 0));
                }
                "stream" => {
                    self.stream.set(true);
                    let seed = x(self, 0);
                    let id = match ann.sid {
                        Some(sid) => sid_hex(sid),
                        None => format!("({}) as u64", x(self, 1)),
                    };
                    return format!("rt::stream({seed}, {id})");
                }
                "uniform" | "normal" | "normal3" => {
                    self.stream.set(true);
                    return format!("rt::{f}(&mut {})", x(self, 0));
                }
                _ => {}
            }
        }
        let (module, name, wants): (&str, &str, Vec<(&Ty, bool)>) = match ann.target {
            Target::Fn(fi) => {
                let f = &c.fns[fi];
                if a.len() > f.params.len() {
                    return self.b.fail(format!("{}: more inputs than it takes", f.name));
                }
                (&f.module, &f.name, f.params.iter().map(|p| (&p.ty, p.inout)).collect())
            }
            Target::Table(ti) => {
                let t = &c.tables[ti];
                (&t.module, &t.name, a.iter().map(|_| (&t.key.ty, false)).collect())
            }
            _ => return self.b.fail("a call of nothing"),
        };
        // an inout input is handed over by reference
        let args: Vec<String> = a.iter().zip(wants).map(|(&x, (w, io))| if io { format!("&mut {}", self.ex(x)) } else { self.e_want(x, Some(w)) }).collect();
        format!("{}::{module}::{name}({})", self.root, args.join(", "))
    }

    fn lv(&mut self, l: usize) -> String {
        let c = self.b.c;
        match &c.exprs[l].kind {
            ExprKind::Var(name) => match c.ann[l].var {
                VarKind::State => format!("st.{name}"),
                VarKind::InOut => format!("(*{name})"),
                _ => name.clone(),
            },
            ExprKind::Index { a, i } => {
                let base = self.lv(*a);
                format!("{base}[{}]", self.idx(*i))
            }
            ExprKind::Field { a, f } => format!("{}.{f}", self.lv(*a)),
            _ => self.b.fail("not a place to set"),
        }
    }

    fn stmts(&mut self, body: &[Stmt], ind: usize) -> String {
        body.iter().map(|s| self.stmt(s, ind)).collect()
    }
    fn stmt(&mut self, s: &Stmt, ind: usize) -> String {
        let c = self.b.c;
        let ii = "    ".repeat(ind);
        match &s.kind {
            StmtKind::Let { names, e, .. } => {
                if names.len() > 1 {
                    let ns: Vec<String> = names.iter().map(|n| format!("mut {n}")).collect();
                    return format!("{ii}let ({}) = {};\n", ns.join(", "), self.ex(*e));
                }
                let Some(vty) = c.ann[*e].vty.as_ref() else { return self.b.fail("a let with no type") };
                let t = self.rty(vty);
                format!("{ii}let mut {}: {t} = {};\n", names[0], self.e_want(*e, Some(vty)))
            }
            StmtKind::State { .. } => String::new(),
            StmtKind::Set { targets, e } => {
                if targets.len() > 1 {
                    let n = format!("__t{}", self.tmp);
                    self.tmp += 1;
                    let mut out = format!("{ii}let {n} = {};\n", self.ex(*e));
                    for (i, t) in targets.iter().enumerate() {
                        out += &format!("{ii}{} = {n}.{i};\n", self.lv(*t));
                    }
                    return out;
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
                    out += &format!("{lead} {cs} {{\n{}{ii}}}", self.stmts(body, ind + 1));
                }
                if let Some(els) = els {
                    out += &format!(" else {{\n{}{ii}}}", self.stmts(els, ind + 1));
                }
                out + "\n"
            }
            StmtKind::For { v, a, b, body } => {
                let n = format!("__end{}", self.tmp);
                self.tmp += 1;
                let bs = self.ex(*b);
                let as_ = self.ex(*a);
                let inner = self.stmts(body, ind + 2);
                format!("{ii}{{\n{ii}    let {n}: i64 = {bs};\n{ii}    let mut {v}: i64 = {as_};\n{ii}    while {v} < {n} {{\n{inner}{ii}        {v} += 1;\n{ii}    }}\n{ii}}}\n")
            }
            StmtKind::Settle { n: ne, c: ce, body, els } => {
                let k = format!("__k{}", self.tmp);
                self.tmp += 1;
                let n = format!("__n{}", self.tmp);
                self.tmp += 1;
                let ns = self.ex(*ne);
                let inner = self.stmts(body, ind + 2);
                let cs = self.ex(*ce);
                let es = match els {
                    Some(els) => self.stmts(els, ind + 3),
                    None => String::new(),
                };
                format!(
                    "{ii}{{\n{ii}    let {n}: i64 = {ns};\n{ii}    let mut {k}: i64 = 0;\n{ii}    loop {{\n{inner}{ii}        {k} += 1;\n{ii}        if {cs} {{ break; }}\n{ii}        if {k} >= {n} {{\n{es}{ii}            break;\n{ii}        }}\n{ii}    }}\n{ii}}}\n"
                )
            }
        }
    }

    fn run(&mut self, title: Option<&str>) -> Files {
        let c = self.b.c;
        let head = head();
        let mut files = Files::new();
        let doc = |lines: &[String]| -> String { lines.iter().map(|l| format!("{}\n", trim_end_js(&format!("/// {l}")))).collect() };
        // a record (a Rust struct) may be named in another module's code: bring every other module's
        // records into scope (names are unique across the program, so nothing clashes)
        let records_of = |mi: usize| -> Vec<&str> {
            c.mod_items[mi].iter().filter_map(|r| if let ItemRef::Record(ri) = r { Some(c.records[*ri].name.as_str()) } else { None }).collect()
        };
        for (mi, (mname, mdoc)) in c.modules.iter().enumerate() {
            let uses: String = (0..c.modules.len())
                .filter(|&o| o != mi && !records_of(o).is_empty())
                .map(|o| format!("use {}::{}::{{{}}};\n", self.root, c.modules[o].0, records_of(o).join(", ")))
                .collect();
            let what = mdoc.join(" ");
            let what = if what.is_empty() { format!("the pseudocode module {mname}") } else { what };
            let mut out = format!(
                "//! {mname}: {what}\n//! {head}\n#![allow(unused_mut, unused_variables, unused_parens, unused_assignments, unused_imports, unreachable_code, non_snake_case, clippy::all)]\nuse {}::rt;\n{uses}\n",
                self.root
            );
            for item in &c.mod_items[mi] {
                match *item {
                    ItemRef::Const(ci) => {
                        let k = &c.consts[ci];
                        let val = self.b.const_of(k.e);
                        let t = self.b.ty_of(k.e);
                        let rt = self.rty(t);
                        let vl = self.value_lit(&val, t);
                        out += &format!("{}pub const {}: {rt} = {vl};\n\n", doc(&k.doc), upper(&k.name));
                    }
                    ItemRef::Choice(ci) => {
                        // its options as whole numbers, from 0
                        let ch = &c.choices[ci];
                        let names: Vec<&str> = ch.options.iter().map(|o| o.0.as_str()).collect();
                        let consts: String = ch.options.iter().enumerate().map(|(i, o)| format!("pub const {}_{}: i64 = {i};\n", upper(&ch.name), upper(&o.0))).collect();
                        out += &format!("{}/// Choice {}: {}.\n{consts}\n", doc(&ch.doc), ch.name, names.join(", "));
                    }
                    ItemRef::Data(di) => {
                        // one copy, indexed in place: a row a line (a 1-D table ten values a line)
                        let d = &c.data[di];
                        let (rows, vals, el) = data_rows(d);
                        let lines: String = match rows {
                            Some(rows) => rows
                                .iter()
                                .map(|r| format!("    [{}],\n", r.iter().map(|x| self.value_lit(&Value::Num(*x), &el)).collect::<Vec<_>>().join(", ")))
                                .collect(),
                            None => vals
                                .chunks(10)
                                .map(|ch| format!("    {},\n", ch.iter().map(|x| self.value_lit(&Value::Num(*x), &el)).collect::<Vec<_>>().join(", ")))
                                .collect(),
                        };
                        let rt = self.rty(&d.ty);
                        out += &format!(
                            "{}/// Data: {}; SI: {}.\npub static DATA_{}: {rt} = [\n{lines}];\n\n",
                            doc(&d.doc),
                            decl_text(&d.decl),
                            si_of(&d.ty),
                            upper(&d.name)
                        );
                    }
                    ItemRef::Record(ri) => {
                        let r = &c.records[ri];
                        let fields: String = r.fields.iter().map(|f| format!("    pub {}: {},\n", f.name, self.rty(&f.ty))).collect();
                        out += &format!("{}#[derive(Clone, Copy, Debug, Default, PartialEq)]\npub struct {} {{\n{fields}}}\n\n", doc(&r.doc), r.name);
                    }
                    ItemRef::Table(ti) => {
                        let t = &c.tables[ti];
                        let (cn, rn) = (t.outs.len(), t.si.len());
                        let tn = format!("TABLE_{}", upper(&t.name));
                        let rows: String = t
                            .si
                            .iter()
                            .map(|r| {
                                let xs: Vec<String> = r.iter().map(|x| self.b.lit(*x)).collect();
                                format!("    [{}],\n", xs.join(", "))
                            })
                            .collect();
                        out += &format!("const {tn}: [[f64; {cn}]; {rn}] = [\n{rows}];\n");
                        let ret = if cn == 1 { "f64".to_string() } else { format!("({})", vec!["f64"; cn].join(", ")) };
                        let outs: Vec<&str> = t.outs.iter().map(|o| o.name.as_str()).collect();
                        let mode = mode_text(t.mode);
                        let key = &t.key.name;
                        let give = if cn == 1 { "r[0]".to_string() } else { format!("({})", (0..cn).map(|i| format!("r[{i}]")).collect::<Vec<_>>().join(", ")) };
                        out += &format!(
                            "{}/// Table ({mode}): {key} -> {}.\npub fn {}({key}: f64) -> {ret} {{\n    let r = rt::lookup_{mode}(&{tn}, {key});\n    {give}\n}}\n\n",
                            doc(&t.doc),
                            outs.join(", "),
                            t.name
                        );
                    }
                    ItemRef::Fn(fi) => {
                        let f = &c.fns[fi];
                        let mut ins: Vec<String> = f.params.iter().map(|p| format!("{}: {}{}", p.name, if p.inout { "&mut " } else { "" }, self.rty(&p.ty))).collect();
                        let mut st_decl = String::new();
                        if f.kind == FnKind::Proc {
                            let s = format!("{}State", self.b.pascal(&f.name));
                            let mut fields = String::new();
                            for (name, e) in &f.states {
                                let t = c.ann[*e].vty.as_ref().unwrap_or(&REAL);
                                fields += &format!("    pub {name}: {},\n", self.rty(t));
                            }
                            let mut starts = String::new();
                            for (name, e) in &f.states {
                                let t = c.ann[*e].vty.as_ref().unwrap_or(&REAL);
                                let v = self.b.const_of(*e);
                                starts += &format!("            {name}: {},\n", self.value_lit(&v, t));
                            }
                            st_decl = format!(
                                "/// The state {} keeps between calls.\n#[derive(Clone, Copy, Debug, PartialEq)]\npub struct {s} {{\n{fields}}}\nimpl Default for {s} {{\n    fn default() -> Self {{\n        {s} {{\n{starts}        }}\n    }}\n}}\n\n",
                                f.name
                            );
                            ins.insert(0, format!("st: &mut {s}"));
                        }
                        let ret = if f.outs.len() == 1 {
                            self.rty(&f.outs[0].ty)
                        } else {
                            let ts: Vec<String> = f.outs.iter().map(|o| self.rty(&o.ty)).collect();
                            format!("({})", ts.join(", "))
                        };
                        let pdoc = doc(&f.params.iter().map(|p| format!("- {}", io(p))).collect::<Vec<_>>());
                        let odoc = doc(&f.outs.iter().map(|o| format!("- returns {}", io(o))).collect::<Vec<_>>());
                        let lets: String = f.outs.iter().map(|o| format!("    let mut {}: {} = {};\n", o.name, self.rty(&o.ty), self.zero(&o.ty))).collect();
                        let body = self.stmts(&f.body, 1);
                        let give = if f.outs.len() == 1 { f.outs[0].name.clone() } else { format!("({})", f.outs.iter().map(|o| o.name.as_str()).collect::<Vec<_>>().join(", ")) };
                        out += &format!(
                            "{st_decl}{}{pdoc}{odoc}#[allow(clippy::too_many_arguments)]\npub fn {}({}) -> {ret} {{\n{lets}{body}    {give}\n}}\n\n",
                            doc(&f.doc),
                            f.name,
                            ins.join(", ")
                        );
                    }
                }
            }
            set_file(&mut files, format!("src/{mname}.rs"), format!("{}\n", trim_end_js(&out)));
        }
        // the dispatcher the vector test calls: every fn whose inputs and outputs are numbers or arrays of them
        // a record is passed field by field in declaration order, as the vectors flatten it
        let mut disp = format!(
            "//! The vector dispatcher: call a function by name with its inputs flattened (SI). {head}\n#![allow(unused_mut, unused_variables, unreachable_code, clippy::all)]\n\n/// The outputs, flattened, or None when there is no such function or too few inputs.\npub fn call(name: &str, x: &[f64]) -> Option<Vec<f64>> {{\n    let mut out = Vec::new();\n    match name {{\n"
        );
        for f in c.fns.iter().filter(|f| f.kind == FnKind::Fn) {
            let mut at = 0;
            let mut args = Vec::new();
            for p in &f.params {
                let (s, n) = self.read_arg(&p.ty, at);
                args.push(s);
                at += n;
            }
            let outs: Vec<String> = if f.outs.len() == 1 { vec!["r".into()] } else { (0..f.outs.len()).map(|i| format!("r.{i}")).collect() };
            disp += &format!(
                "        \"{0}::{1}\" => {{\n            if x.len() != {at} {{ return None; }}\n{3}            let r = crate::{0}::{1}({2});\n",
                f.module,
                f.name,
                io_args(f, &args).join(", "),
                io_lets(f, &args, "            ")
            );
            for (o, r) in f.outs.iter().zip(&outs) {
                disp += &format!("            {}\n", self.push_out(&o.ty, r));
            }
            disp += &self.io_push(f, "            ");
            disp += "        }\n";
        }
        disp += "        _ => return None,\n    }\n    Some(out)\n}\n";
        disp += "\n/// A proc called once per entry of `calls`, its state carried from each call to the next.\npub fn call_seq(name: &str, calls: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {\n    let mut outs = Vec::new();\n    match name {\n";
        for f in c.fns.iter().filter(|f| f.kind == FnKind::Proc) {
            let mut at = 0;
            let mut args = Vec::new();
            for p in &f.params {
                let (s, n) = self.read_arg(&p.ty, at);
                args.push(s);
                at += n;
            }
            let outs: Vec<String> = if f.outs.len() == 1 { vec!["r".into()] } else { (0..f.outs.len()).map(|i| format!("r.{i}")).collect() };
            let s = self.b.pascal(&f.name);
            disp += &format!(
                "        \"{0}::{1}\" => {{\n            let mut st = crate::{0}::{s}State::default();\n            for x in calls {{\n                if x.len() != {at} {{ return None; }}\n                let mut out = Vec::new();\n{3}                let r = crate::{0}::{1}(&mut st, {2});\n",
                f.module,
                f.name,
                io_args(f, &args).join(", "),
                io_lets(f, &args, "                ")
            );
            for (o, r) in f.outs.iter().zip(&outs) {
                disp += &format!("                {}\n", self.push_out(&o.ty, r));
            }
            disp += &self.io_push(f, "                ");
            disp += "                outs.push(out);\n            }\n        }\n";
        }
        disp += "        _ => return None,\n    }\n    Some(outs)\n}\n";
        if self.dispatch {
            set_file(&mut files, "src/dispatch.rs".into(), disp);
        }
        let rt = match &self.math {
            Some(m) => RUST_RT.replacen("dot(a, a).sqrt()", &format!("{m}::sqrt(dot(a, a))"), 1),
            None => RUST_RT.to_string(),
        };
        let sort = if self.sort { RUST_RT_SORT } else { "" };
        let stream = if self.stream.get() { rust_rt_stream(self.math.as_deref()) } else { String::new() };
        let erf = if self.erf { RUST_RT_ERF } else { "" };
        set_file(&mut files, "src/rt.rs".into(), format!("//! The arithmetic every translation shares with the interpreter (design/js/pcode.js `rt`). {head}\n{rt}{sort}{stream}{erf}"));
        let title = title.filter(|t| !t.is_empty()).unwrap_or("Functions written in the pseudocode");
        let mods: String = c.modules.iter().map(|(m, _)| format!("pub mod {m};\n")).collect();
        set_file(
            &mut files,
            if self.root == "crate" { "src/lib.rs" } else { "src/mod.rs" }.into(),
            format!(
                "//! {title}. {head}\n//! Every relation is SI in and SI out; each function's doc lists its inputs and outputs with their units.\n#![allow(clippy::all)]\npub mod rt;\n{}{mods}",
                if self.dispatch { "pub mod dispatch;\n" } else { "" }
            ),
        );
        files
    }

    /// an inout input's value after the call, pushed after the outputs
    fn io_push(&self, f: &crate::ast::Func, ii: &str) -> String {
        f.params.iter().enumerate().filter(|(_, p)| p.inout).map(|(i, p)| format!("{ii}{}\n", self.push_out(&p.ty, &format!("io{i}")))).collect()
    }
    fn read_arg(&self, t: &Ty, at: usize) -> (String, usize) {
        match t {
            Ty::Stream => {
                self.stream.set(true);
                (format!("crate::rt::Stream::from_words([{}])", (0..6).map(|k| format!("x[{}]", at + k)).collect::<Vec<_>>().join(", ")), 6)
            }
            Ty::Real(_) => (format!("x[{at}]"), 1),
            Ty::Int | Ty::Choice(_) => (format!("x[{at}] as i64"), 1),
            Ty::Bool => (format!("x[{at}] != 0.0"), 1),
            Ty::Rec(name) => {
                let (mut parts, mut k) = (Vec::new(), 0);
                let Some(r) = self.b.record(name) else { return (String::new(), 0) };
                for f in &r.fields {
                    let (s, n) = self.read_arg(&f.ty, at + k);
                    parts.push(format!("{}: {s}", f.name));
                    k += n;
                }
                (format!("crate::{}::{name} {{ {} }}", r.module, parts.join(", ")), k)
            }
            // a long array of numbers (a workspace, a table) as a loop, not an expression an element
            Ty::Arr(n, of) if *n > 1024 && matches!(**of, Ty::Real(_) | Ty::Int | Ty::Choice(_) | Ty::Bool) => {
                let x = format!("x[{at} + i]");
                let (z, v) = match **of {
                    Ty::Real(_) => ("0.0".to_string(), x),
                    Ty::Bool => ("false".to_string(), format!("{x} != 0.0")),
                    _ => ("0i64".to_string(), format!("{x} as i64")),
                };
                (format!("{{ let mut a = [{z}; {n}]; for i in 0..{n} {{ a[i] = {v}; }} a }}"), *n)
            }
            Ty::Arr(n, of) => {
                let (mut parts, mut k) = (Vec::new(), 0);
                for _ in 0..*n {
                    let (s, m) = self.read_arg(of, at + k);
                    parts.push(s);
                    k += m;
                }
                (format!("[{}]", parts.join(", ")), k)
            }
            Ty::Tuple(_) | Ty::Str => ("[]".into(), 0),
        }
    }
    fn push_out(&self, t: &Ty, name: &str) -> String {
        match t {
            Ty::Stream => format!("for v in {name}.words().iter() {{ let v = *v; out.push(v); }}"),
            Ty::Real(_) => format!("out.push({name});"),
            Ty::Int | Ty::Choice(_) => format!("out.push({name} as f64);"),
            Ty::Bool => format!("out.push(if {name} {{ 1.0 }} else {{ 0.0 }});"),
            Ty::Rec(rn) => match self.b.record(rn) {
                Some(r) => r.fields.iter().map(|f| self.push_out(&f.ty, &format!("{name}.{}", f.name))).collect::<Vec<_>>().join(" "),
                None => String::new(),
            },
            Ty::Arr(_, of) => format!("for v in {name}.iter() {{ let v = *v; {} }}", self.push_out(of, "v")),
            Ty::Tuple(_) | Ty::Str => format!("for v in {name}.iter() {{ let v = *v; undefined }}"),
        }
    }
}

/// an inout input of the dispatcher's call: read into a variable of its own, handed over by reference
fn io_lets(f: &crate::ast::Func, args: &[String], ii: &str) -> String {
    f.params.iter().zip(args).enumerate().filter(|(_, (p, _))| p.inout).map(|(i, (_, a))| format!("{ii}let mut io{i} = {a};\n")).collect()
}
fn io_args(f: &crate::ast::Func, args: &[String]) -> Vec<String> {
    f.params.iter().zip(args).enumerate().map(|(i, (p, a))| if p.inout { format!("&mut io{i}") } else { a.clone() }).collect()
}

/// A random stream (pcode_gen.js `rustRtStream`): adcs-sim-core rng.rs, value for value; carried by a translation that draws.
fn rust_rt_stream(m: Option<&str>) -> String {
    let f = |std: &str, name: &str| match m {
        Some(m) => format!("{m}::{name}"),
        None => format!("f64::{std}"),
    };
    format!(
        r#"/// A counter-based random stream (adcs-sim-core rng.rs): SplitMix64 over a counter, normal draws by Box-Muller.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stream {{ pub key: u64, pub n: u64, pub spare: f64, pub has: bool }}
impl Stream {{
    /// From its six numbers (the key's and the counter's high and low 32 bits, the spare, whether there is one).
    pub fn from_words(w: [f64; 6]) -> Stream {{ Stream {{ key: ((w[0] as u64) << 32) | (w[1] as u64), n: ((w[2] as u64) << 32) | (w[3] as u64), spare: w[4], has: w[5] != 0.0 }} }}
    pub fn words(&self) -> [f64; 6] {{ [(self.key >> 32) as f64, (self.key & 0xFFFF_FFFF) as f64, (self.n >> 32) as f64, (self.n & 0xFFFF_FFFF) as f64, self.spare, if self.has {{ 1.0 }} else {{ 0.0 }}] }}
}}
pub fn splitmix64(mut z: u64) -> u64 {{
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}}
/// The stream of a seed and an id.
pub fn stream(seed: i64, id: u64) -> Stream {{ Stream {{ key: splitmix64((seed as u64) ^ splitmix64(id)), n: 0, spare: 0.0, has: false }} }}
/// Uniform on (0, 1).
pub fn uniform(s: &mut Stream) -> f64 {{ s.n = s.n.wrapping_add(1); let z = splitmix64(s.key ^ splitmix64(s.n)); ((z >> 11) as f64 + 0.5) * (1.0 / 9007199254740992.0) }}
/// Normal, by Box-Muller, the spare kept for the next draw.
pub fn normal(s: &mut Stream) -> f64 {{
    if s.has {{ s.has = false; let v = s.spare; s.spare = 0.0; return v; }}
    let u1 = uniform(s);
    let u2 = uniform(s);
    let r = {sqrt}(-2.0 * {ln}(u1));
    s.spare = r * {sin}(2.0 * core::f64::consts::PI * u2);
    s.has = true;
    r * {cos}(2.0 * core::f64::consts::PI * u2)
}}
pub fn normal3(s: &mut Stream) -> [f64; 3] {{ let a = normal(s); let b = normal(s); let c = normal(s); [a, b, c] }}
"#,
        sqrt = f("sqrt", "sqrt"),
        ln = f("ln", "log"),
        sin = f("sin", "sin"),
        cos = f("cos", "cos")
    )
}

/// The platform's erf (pcode_gen.js `RUST_RT_ERF`), carried by a translation with std's maths that calls it.
const RUST_RT_ERF: &str = r#"extern "C" {
    #[link_name = "erf"]
    fn c_erf(x: f64) -> f64;
}
/// The error function of the platform's C maths library, the one C and MATLAB call (Rust's std has none).
pub fn erf(x: f64) -> f64 {
    // SAFETY: erf is a pure C99 <math.h> function of one double, in the system libm std already links.
    unsafe { c_erf(x) }
}
"#;

/// The toolbox sort (pcode_gen.js `RUST_RT_SORT`), carried by a translation that sorts.
const RUST_RT_SORT: &str = r#"/// The indices of a vector in ascending order, equal values in their order (an insertion sort).
pub fn argsort<T: PartialOrd + Copy, const N: usize>(v: [T; N]) -> [i64; N] {
    let mut ix = [0i64; N];
    for i in 0..N { ix[i] = i as i64; }
    for i in 1..N {
        let k = ix[i];
        let mut j = i;
        while j > 0 && v[ix[j - 1] as usize] > v[k as usize] { ix[j] = ix[j - 1]; j -= 1; }
        ix[j] = k;
    }
    ix
}
/// The vector in ascending order, equal values in their order.
pub fn sort<T: PartialOrd + Copy, const N: usize>(v: [T; N]) -> [T; N] { let ix = argsort(v); let mut r = v; for i in 0..N { r[i] = v[ix[i] as usize]; } r }
"#;

/// The runtime every Rust translation carries (pcode_gen.js `RUST_RT`, after its first line).
const RUST_RT: &str = r#"#![allow(clippy::all)]

/// x^k by repeated multiplication, left to right; x^-k = 1/(x^k).
pub fn ipow(x: f64, k: i32) -> f64 {
    if k == 0 { return 1.0; }
    let mut r = x;
    let mut i = 1;
    while i < k.abs() { r = r * x; i += 1; }
    if k < 0 { 1.0 / r } else { r }
}
pub fn ipowi(x: i64, k: i32) -> i64 { let mut r: i64 = 1; for _ in 0..k { r *= x; } r }
pub fn fmin(a: f64, b: f64) -> f64 { if b < a { b } else { a } }
pub fn fmax(a: f64, b: f64) -> f64 { if b > a { b } else { a } }
pub fn imin(a: i64, b: i64) -> i64 { if b < a { b } else { a } }
pub fn imax(a: i64, b: i64) -> i64 { if b > a { b } else { a } }
pub fn fabs(x: f64) -> f64 { if x < 0.0 { -x } else if x == 0.0 { 0.0 } else { x } }
pub fn clamp(x: f64, lo: f64, hi: f64) -> f64 { fmin(fmax(x, lo), hi) }
pub fn sign(x: f64) -> f64 { if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 } }
pub fn dot<const N: usize>(a: [f64; N], b: [f64; N]) -> f64 { let mut s = a[0] * b[0]; for i in 1..N { s = s + a[i] * b[i]; } s }
pub fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
pub fn norm<const N: usize>(a: [f64; N]) -> f64 { dot(a, a).sqrt() }
pub fn unit<const N: usize>(a: [f64; N]) -> [f64; N] { let n = fmax(norm(a), 1e-30); let mut r = a; for i in 0..N { r[i] = a[i] / n; } r }
pub fn vadd<const N: usize>(a: [f64; N], b: [f64; N]) -> [f64; N] { let mut r = a; for i in 0..N { r[i] = a[i] + b[i]; } r }
pub fn vsub<const N: usize>(a: [f64; N], b: [f64; N]) -> [f64; N] { let mut r = a; for i in 0..N { r[i] = a[i] - b[i]; } r }
pub fn vneg<const N: usize>(a: [f64; N]) -> [f64; N] { let mut r = a; for i in 0..N { r[i] = -a[i]; } r }
pub fn vscale<const N: usize>(a: [f64; N], s: f64) -> [f64; N] { let mut r = a; for i in 0..N { r[i] = a[i] * s; } r }
pub fn vscale_l<const N: usize>(s: f64, a: [f64; N]) -> [f64; N] { let mut r = a; for i in 0..N { r[i] = s * a[i]; } r }
pub fn vdiv<const N: usize>(a: [f64; N], s: f64) -> [f64; N] { let mut r = a; for i in 0..N { r[i] = a[i] / s; } r }
pub fn madd<const N: usize, const M: usize>(a: [[f64; M]; N], b: [[f64; M]; N]) -> [[f64; M]; N] { let mut r = a; for i in 0..N { r[i] = vadd(a[i], b[i]); } r }
pub fn msub<const N: usize, const M: usize>(a: [[f64; M]; N], b: [[f64; M]; N]) -> [[f64; M]; N] { let mut r = a; for i in 0..N { r[i] = vsub(a[i], b[i]); } r }
pub fn mneg<const N: usize, const M: usize>(a: [[f64; M]; N]) -> [[f64; M]; N] { let mut r = a; for i in 0..N { r[i] = vneg(a[i]); } r }
pub fn mscale<const N: usize, const M: usize>(a: [[f64; M]; N], s: f64) -> [[f64; M]; N] { let mut r = a; for i in 0..N { r[i] = vscale(a[i], s); } r }
pub fn mscale_l<const N: usize, const M: usize>(s: f64, a: [[f64; M]; N]) -> [[f64; M]; N] { let mut r = a; for i in 0..N { r[i] = vscale_l(s, a[i]); } r }
pub fn mdiv<const N: usize, const M: usize>(a: [[f64; M]; N], s: f64) -> [[f64; M]; N] { let mut r = a; for i in 0..N { r[i] = vdiv(a[i], s); } r }
pub fn mv<const N: usize, const M: usize>(a: [[f64; M]; N], v: [f64; M]) -> [f64; N] { let mut r = [0.0; N]; for i in 0..N { r[i] = dot(a[i], v); } r }
pub fn mm<const N: usize, const K: usize, const M: usize>(a: [[f64; K]; N], b: [[f64; M]; K]) -> [[f64; M]; N] {
    let mut r = [[0.0; M]; N];
    for i in 0..N { for j in 0..M { let mut s = a[i][0] * b[0][j]; for k in 1..K { s = s + a[i][k] * b[k][j]; } r[i][j] = s; } }
    r
}
pub fn tr<const N: usize, const M: usize>(a: [[f64; M]; N]) -> [[f64; N]; M] { let mut r = [[0.0; N]; M]; for i in 0..N { for j in 0..M { r[j][i] = a[i][j]; } } r }
/// The last row whose key is at or below x (the first row below the first key).
pub fn lookup_step<const C: usize, const R: usize>(t: &[[f64; C]; R], x: f64) -> [f64; C] {
    let mut i = 0;
    while i + 1 < R && t[i + 1][0] <= x { i += 1; }
    t[i]
}
/// Straight-line between the two rows around x, held at the ends.
pub fn lookup_linear<const C: usize, const R: usize>(t: &[[f64; C]; R], x: f64) -> [f64; C] {
    let xc = clamp(x, t[0][0], t[R - 1][0]);
    let mut i = 0;
    while i + 2 < R && t[i + 1][0] <= xc { i += 1; }
    let s = (xc - t[i][0]) / (t[i + 1][0] - t[i][0]);
    let mut r = t[i];
    for j in 0..C { r[j] = t[i][j] + s * (t[i + 1][j] - t[i][j]); }
    r
}
"#;
