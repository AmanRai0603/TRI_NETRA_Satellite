//! The translator to MATLAB: design/js/pcode_gen.js `toMatlab` and `matlabRuntime`, line for
//! line. A function a file in its module's package folder, a record a function making its zero,
//! a proc a function that is handed its state and gives it back; arrays are columns, a matrix's
//! rows are its rows.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

use super::*;
use crate::ast::{BinOp, ExprKind, FnKind, Stmt, StmtKind, TableMode};
use crate::check::{ItemRef, Shape, Target, VarKind};

/// the shared runtime's package (`opts.rt`)
const RTP: &str = "asils.pc";
/// the loop's values, in a guard read at once, until the loop's vector is named (toMatlab's VMARK)
const VMARK: &str = "V__LOOP";

pub(super) fn to_matlab(i: &Interp, pkg: Option<&str>) -> Result<Files, String> {
    let pkg = pkg.filter(|p| !p.is_empty()).unwrap_or("asils.relations").to_string();
    let mut g = Gen { b: Base::new(i, ["Inf", "-Inf", "NaN"]), pkg, tmp: 0 };
    let files = g.run();
    g.b.done(files)
}

struct Gen<'a> {
    b: Base<'a>,
    pkg: String,
    tmp: usize,
}

fn mode_text(m: TableMode) -> &'static str {
    match m {
        TableMode::Step => "step",
        TableMode::Linear => "linear",
    }
}

/// `lit(x)` with no `.0`: MATLAB's literal
fn ml(b: &mut Base, x: f64) -> String {
    let s = b.lit(x);
    let s = match s.strip_suffix(".0") {
        Some(t) => t.to_string(),
        None => s,
    };
    s.replacen(".0e", "e", 1)
}

/// the help block of a file: its name and first line, then the rest
fn help(name: &str, lines: &[String]) -> String {
    match lines.split_first() {
        Some((first, rest)) => {
            let mut s = format!("%{}  {first}\n", upper(name));
            for l in rest {
                s += trim_end_js(&format!("%   {l}"));
                s.push('\n');
            }
            s
        }
        None => format!("%{}\n", upper(name)),
    }
}

impl<'a> Gen<'a> {
    fn zero(&self, t: &Ty) -> String {
        match t {
            Ty::Real(_) | Ty::Int | Ty::Choice(_) => "0".into(),
            Ty::Bool => "false".into(),
            Ty::Arr(n, of) => match &**of {
                Ty::Arr(m, _) => format!("zeros({n}, {m})"),
                _ => format!("zeros({n}, 1)"),
            },
            Ty::Rec(name) => {
                let module = self.b.record(name).map_or("undefined", |r| r.module.as_str());
                format!("{}.{module}.{name}_zero()", self.pkg)
            }
            Ty::Stream => "zeros(6, 1)".into(),
            Ty::Tuple(_) | Ty::Str | Ty::Buf(_) => "undefined".into(),
        }
    }
    fn value_lit(&mut self, v: &Value, t: &Ty) -> String {
        match t {
            Ty::Arr(_, of) => {
                let Value::Arr(items) = v else { return self.b.fail("v.map is not a function") };
                if matches!(**of, Ty::Arr(..)) {
                    let rows: Vec<String> = items
                        .iter()
                        .map(|r| match r {
                            Value::Arr(xs) => xs.iter().map(|x| self.b.lit_value(x)).collect::<Vec<_>>().join(", "),
                            _ => self.b.fail("r.map is not a function"),
                        })
                        .collect();
                    format!("[{}]", rows.join("; "))
                } else {
                    let xs: Vec<String> = items.iter().map(|x| self.value_lit(x, of)).collect();
                    format!("[{}]", xs.join("; "))
                }
            }
            Ty::Bool | Ty::Int | Ty::Choice(_) => js_string(v),
            Ty::Stream => {
                let Value::Arr(items) = v else { return self.b.fail("v.map is not a function") };
                let ws: Vec<String> = items.iter().map(|x| ml(&mut self.b, x.num())).collect();
                format!("[{}]", ws.join("; "))
            }
            _ => self.b.lit_value(v),
        }
    }
    fn value_lit_m(&mut self, v: &Value, t: &Ty) -> String {
        if let (Ty::Real(_), Value::Num(x)) = (t, v) {
            let s = ml(&mut self.b, *x);
            return if *x < 0.0 { format!("({s})") } else { s };
        }
        if let Ty::Real(_) = t {
            return self.b.lit_value(v);
        }
        self.value_lit(v, t)
    }

    /// a data table named by the expression: its function (`pkg.module.NAME`)
    fn data_ref(&self, x: usize) -> Option<String> {
        let c = self.b.c;
        match (&c.exprs[x].kind, c.ann[x].var) {
            (ExprKind::Var(name), VarKind::Data(di)) => Some(format!("{}.{}.{name}", self.pkg, c.data[di].module)),
            _ => None,
        }
    }
    fn idx(&mut self, i: usize) -> String {
        match &self.b.c.exprs[i].kind {
            ExprKind::Num { v, .. } => jsfmt::num(v + 1.0),
            _ => format!("({}) + 1", self.ex(i)),
        }
    }
    /// is `a` (an index's base) itself an index into a matrix? then the two read as one
    fn matrix_pair(&self, a: usize) -> Option<(usize, usize)> {
        let c = self.b.c;
        if let ExprKind::Index { a: aa, i: ai } = &c.exprs[a].kind {
            if let Some(Ty::Arr(_, of)) = c.ann[*aa].ty.as_ref() {
                if matches!(**of, Ty::Arr(..)) {
                    return Some((*aa, *ai));
                }
            }
        }
        None
    }
    /// is the type of `a` an array of arrays (a matrix, so an index of it is a row)?
    fn rows_of(&mut self, a: usize) -> bool {
        match self.b.ty_of(a) {
            Ty::Arr(_, of) | Ty::Buf(of) => matches!(**of, Ty::Arr(..)),
            _ => {
                self.b.fail("cannot read properties of undefined (reading 'k')");
                false
            }
        }
    }

    // Read in place (speed, the same arithmetic): a small vector or matrix the expression names, whose elements can be
    // read where they are (a variable, a record's field of one, a row of a matrix variable or data table, its index free
    // of calls): the text of its elements, else none. dot, norm, cross and the matrix-vector product of such operands are
    // written out term by term, summed left to right as the runtime's dot_ and mv sum them (toMatlab's elems, melems).
    fn no_call(&self, e: usize) -> bool {
        let c = self.b.c;
        match &c.exprs[e].kind {
            ExprKind::Call { .. } => false,
            ExprKind::Num { .. } | ExprKind::Bool(_) | ExprKind::Var(_) | ExprKind::Str(_) => true,
            ExprKind::Arr { items, .. } => items.iter().all(|&x| self.no_call(x)),
            ExprKind::Index { a, i } => self.no_call(*a) && self.no_call(*i),
            ExprKind::Field { a, .. } | ExprKind::Not(a) | ExprKind::Neg(a) => self.no_call(*a),
            ExprKind::Ifx { c: cc, a, b } => self.no_call(*a) && self.no_call(*b) && self.no_call(*cc),
            ExprKind::Bin { a, b, .. } => self.no_call(*a) && self.no_call(*b),
        }
    }
    fn plain_base(&self, e: usize) -> bool {
        let c = self.b.c;
        match &c.exprs[e].kind {
            ExprKind::Var(_) => !matches!(c.ann[e].var, VarKind::Const(_) | VarKind::Data(_) | VarKind::Pi | VarKind::Inf | VarKind::Nan),
            ExprKind::Field { a, .. } => c.ann[e].choice.is_none() && !matches!(c.exprs[*a].kind, ExprKind::Call { .. }) && self.no_call(*a),
            _ => false,
        }
    }
    fn elems(&mut self, e: usize, n: usize) -> Option<Vec<String>> {
        let c = self.b.c;
        match c.ann[e].ty.as_ref() {
            Some(Ty::Arr(m, of)) if *m == n && !matches!(**of, Ty::Arr(..)) => {}
            _ => return None,
        }
        if self.plain_base(e) {
            let b = self.ex(e);
            return Some((1..=n).map(|k| format!("{b}({k})")).collect());
        }
        if let ExprKind::Index { a, i } = &c.exprs[e].kind {
            let rows = matches!(c.ann[*a].ty.as_ref(), Some(Ty::Arr(_, of)) if matches!(**of, Ty::Arr(..)));
            if rows && self.no_call(*i) && (self.plain_base(*a) || self.data_ref(*a).is_some()) {
                let m = match self.data_ref(*a) {
                    Some(d) => d,
                    None => self.ex(*a),
                };
                let r = self.idx(*i);
                return Some((1..=n).map(|k| format!("{m}({r}, {k})")).collect());
            }
        }
        None
    }
    fn melems(&mut self, e: usize, n: usize, m: usize) -> Option<String> {
        let c = self.b.c;
        match c.ann[e].ty.as_ref() {
            Some(Ty::Arr(rn, of)) if *rn == n => match &**of {
                Ty::Arr(cm, el) if *cm == m && !matches!(**el, Ty::Arr(..)) => {}
                _ => return None,
            },
            _ => return None,
        }
        if self.plain_base(e) || self.data_ref(e).is_some() {
            return Some(match self.data_ref(e) {
                Some(d) => d,
                None => self.ex(e),
            });
        }
        None
    }
    fn arr_len(&self, e: usize) -> usize {
        match self.b.c.ann[e].ty.as_ref() {
            Some(Ty::Arr(n, _)) => *n,
            _ => 0,
        }
    }

    // A loop's guard read at once (speed, the same arithmetic): toMatlab's guarded, vcond and setsIn.
    fn sets_in(&self, body: &[Stmt], out: &mut Vec<String>) {
        let c = self.b.c;
        let root = |mut x: usize| -> String {
            loop {
                match &c.exprs[x].kind {
                    ExprKind::Var(n) => return n.clone(),
                    ExprKind::Index { a, .. } | ExprKind::Field { a, .. } => x = *a,
                    _ => return String::new(),
                }
            }
        };
        for st in body {
            match &st.kind {
                StmtKind::Let { names, e, .. } => {
                    out.extend(names.iter().cloned());
                    if c.ann[*e].inout {
                        if let ExprKind::Call { args, .. } = &c.exprs[*e].kind {
                            for &i in &c.ann[*e].inout_args {
                                if let Some(&a) = args.get(i) { out.push(root(a)); }
                            }
                        }
                    }
                }
                StmtKind::Set { targets, e } => {
                    out.extend(targets.iter().map(|&t| root(t)));
                    if c.ann[*e].inout {
                        if let ExprKind::Call { args, .. } = &c.exprs[*e].kind {
                            for &i in &c.ann[*e].inout_args {
                                if let Some(&a) = args.get(i) { out.push(root(a)); }
                            }
                        }
                    }
                }
                StmtKind::State { name, .. } => out.push(name.clone()),
                StmtKind::For { v, body, .. } => {
                    out.push(v.clone());
                    self.sets_in(body, out);
                }
                StmtKind::If { arms, els } => {
                    for (_, b) in arms { self.sets_in(b, out); }
                    if let Some(b) = els { self.sets_in(b, out); }
                }
                StmtKind::Settle { body, els, .. } => {
                    self.sets_in(body, out);
                    if let Some(b) = els { self.sets_in(b, out); }
                }
            }
        }
    }
    fn scalar_t(&self, e: usize) -> bool {
        matches!(self.b.c.ann[e].ty.as_ref(), Some(Ty::Real(_) | Ty::Int | Ty::Bool | Ty::Choice(_)))
    }
    fn kept(&self, x: usize, v: &str, sets: &[String]) -> bool {
        let c = self.b.c;
        match &c.exprs[x].kind {
            ExprKind::Var(n) => n != v && !sets.iter().any(|s| s == n) && !matches!(c.ann[x].var, VarKind::Const(_) | VarKind::Pi | VarKind::Inf | VarKind::Nan),
            _ => false,
        }
    }
    fn vcond(&mut self, e: usize, v: &str, sets: &[String]) -> Option<String> {
        let c = self.b.c;
        match &c.exprs[e].kind {
            ExprKind::Num { .. } | ExprKind::Bool(_) => Some(self.ex(e)),
            ExprKind::Var(n) => {
                if n == v {
                    return Some(VMARK.into());
                }
                if self.kept(e, v, sets) && !matches!(c.ann[e].var, VarKind::Data(_)) && self.scalar_t(e) { Some(self.ex(e)) } else { None }
            }
            ExprKind::Index { a, i } => {
                let vec = matches!(c.ann[*a].ty.as_ref(), Some(Ty::Arr(_, of)) if !matches!(**of, Ty::Arr(..)));
                if !self.scalar_t(e) || !matches!(c.exprs[*a].kind, ExprKind::Var(_)) || !self.kept(*a, v, sets) || !vec {
                    return None;
                }
                let iv = self.vcond(*i, v, sets)?;
                let base = match self.data_ref(*a) {
                    Some(d) => d,
                    None => self.ex(*a),
                };
                Some(format!("{base}(({iv}) + 1)"))
            }
            ExprKind::Field { a, .. } => {
                if c.ann[e].choice.is_none() && self.scalar_t(e) && matches!(c.exprs[*a].kind, ExprKind::Var(_)) && self.kept(*a, v, sets) {
                    Some(self.ex(e))
                } else {
                    None
                }
            }
            ExprKind::Not(a) => self.vcond(*a, v, sets).map(|x| format!("(~{x})")),
            ExprKind::Neg(a) => self.vcond(*a, v, sets).map(|x| format!("(-({x}))")),
            ExprKind::Bin { op, a, b } => {
                use BinOp::*;
                if !matches!(op, Add | Sub | Mul | Div | Eq | Ne | Lt | Le | Gt | Ge) || matches!(c.ann[e].shape, Shape::Mv | Shape::Mm) {
                    return None;
                }
                if !self.scalar_t(*a) || !self.scalar_t(*b) {
                    return None;
                }
                let x = self.vcond(*a, v, sets)?;
                let y = self.vcond(*b, v, sets)?;
                let o = match op {
                    Mul => ".*",
                    Div => "./",
                    Ne => "~=",
                    _ => op.text(),
                };
                Some(format!("({x} {o} {y})"))
            }
            ExprKind::Call { args, .. } => {
                if !matches!(c.ann[e].target, Target::Builtin("dot")) {
                    return None;
                }
                let n = args.first().map_or(0, |&x| self.arr_len(x));
                if !(1..=4).contains(&n) {
                    return None;
                }
                let a0 = self.vside(args[0], n, v, sets)?;
                let a1 = self.vside(*args.get(1)?, n, v, sets)?;
                let t: Vec<String> = (0..n).map(|k| format!("{} .* {}", a0[k], a1[k])).collect();
                Some(format!("({})", t.join(" + ")))
            }
            _ => None,
        }
    }
    fn vside(&mut self, x: usize, n: usize, v: &str, sets: &[String]) -> Option<Vec<String>> {
        let c = self.b.c;
        if let ExprKind::Var(_) = &c.exprs[x].kind {
            let vec = matches!(c.ann[x].ty.as_ref(), Some(Ty::Arr(m, of)) if *m == n && !matches!(**of, Ty::Arr(..)));
            if self.kept(x, v, sets) && !matches!(c.ann[x].var, VarKind::Data(_)) && vec {
                let b = self.ex(x);
                return Some((1..=n).map(|k| format!("{b}({k})")).collect());
            }
        }
        if let ExprKind::Index { a, i } = &c.exprs[x].kind {
            let rows = matches!(c.ann[*a].ty.as_ref(), Some(Ty::Arr(_, of)) if matches!(&**of, Ty::Arr(m, _) if *m == n));
            if matches!(c.exprs[*a].kind, ExprKind::Var(_)) && self.kept(*a, v, sets) && rows {
                let iv = self.vcond(*i, v, sets)?;
                let m = match self.data_ref(*a) {
                    Some(d) => d,
                    None => self.ex(*a),
                };
                return Some((1..=n).map(|k| format!("{m}(({iv}) + 1, {k})")).collect());
            }
        }
        None
    }
    fn guarded(&mut self, v: &str, body: &[Stmt]) -> Option<String> {
        if body.len() != 1 {
            return None;
        }
        let StmtKind::If { arms, els } = &body[0].kind else { return None };
        if arms.len() != 1 || els.is_some() {
            return None;
        }
        let mut sets = Vec::new();
        self.sets_in(body, &mut sets);
        let g = self.vcond(arms[0].0, v, &sets)?;
        if g.contains(VMARK) { Some(g) } else { None }
    }

    // a branch that could fail when it is not the one taken (an index, a call)
    fn may_fail(&self, e: usize) -> bool {
        let c = self.b.c;
        match &c.exprs[e].kind {
            ExprKind::Index { .. } => true,
            ExprKind::Call { args, .. } => match c.ann[e].target {
                Target::Record(_) | Target::Builtin(_) => args.iter().any(|&x| self.may_fail(x)),
                _ => true,
            },
            ExprKind::Num { .. } | ExprKind::Bool(_) | ExprKind::Var(_) | ExprKind::Str(_) => false,
            ExprKind::Arr { items, .. } => items.iter().any(|&x| self.may_fail(x)),
            ExprKind::Field { a, .. } | ExprKind::Not(a) | ExprKind::Neg(a) => self.may_fail(*a),
            ExprKind::Ifx { c: cc, a, b } => self.may_fail(*a) || self.may_fail(*b) || self.may_fail(*cc),
            ExprKind::Bin { a, b, .. } => self.may_fail(*a) || self.may_fail(*b),
        }
    }

    fn ex(&mut self, e: usize) -> String {
        let c = self.b.c;
        let ann = &c.ann[e];
        match &c.exprs[e].kind {
            ExprKind::Num { v, .. } => match &ann.fill {
                Some(f) => self.zero(f),
                None if is_int(ann.ty.as_ref()) => jsfmt::num(*v),
                None => ml(&mut self.b, ann.si),
            },
            ExprKind::Bool(v) => v.to_string(),
            ExprKind::Str(s) => jsfmt::json_str(s),
            ExprKind::Var(name) => match ann.var {
                VarKind::Pi => "pi".into(),
                VarKind::Inf | VarKind::Nan => self.b.non_finite(const_value(ann.var)),
                VarKind::Const(ci) => {
                    let k = &c.consts[ci];
                    let v = self.b.const_of(k.e);
                    let t = self.b.ty_of(k.e);
                    self.value_lit_m(&v, t)
                }
                VarKind::Data(_) => format!("{}()", self.data_ref(e).unwrap_or_default()),
                VarKind::State => format!("st.{name}"),
                _ => name.clone(),
            },
            ExprKind::Arr { items, .. } => {
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
                            ml(&mut self.b, v * scale)
                        })
                        .collect();
                    return format!("[{}]", parts.join("; "));
                }
                if self.rows_of(e) {
                    let parts: Vec<String> = items.iter().map(|&x| format!("({}).'", self.ex(x))).collect();
                    return format!("[{}]", parts.join("; "));
                }
                let parts: Vec<String> = items.iter().map(|&x| self.ex(x)).collect();
                format!("[{}]", parts.join("; "))
            }
            ExprKind::Index { a, i } => {
                // a data table is a function holding its one copy: it is indexed through its call
                if let Some((aa, ai)) = self.matrix_pair(*a) {
                    let m = match self.data_ref(aa) {
                        Some(d) => d,
                        None => self.ex(aa),
                    };
                    let r = self.idx(ai);
                    return format!("{m}({r}, {})", self.idx(*i));
                }
                if self.rows_of(*a) {
                    if let Some(d) = self.data_ref(*a) {
                        return format!("({d}({}, ':')).'", self.idx(*i));
                    }
                    let m = self.ex(*a);
                    return format!("({m}({}, :)).'", self.idx(*i));
                }
                let m = match self.data_ref(*a) {
                    Some(d) => d,
                    None => self.ex(*a),
                };
                format!("{m}({})", self.idx(*i))
            }
            ExprKind::Field { a, f } => match ann.choice {
                Some((_, i)) => i.to_string(),
                // a call's result cannot be indexed in MATLAB (f().x): its field through getfield
                None if matches!(c.exprs[*a].kind, ExprKind::Call { .. }) => format!("getfield({}, '{f}')", self.ex(*a)),
                None => format!("{}.{f}", self.ex(*a)),
            },
            ExprKind::Not(a) => format!("(~{})", self.ex(*a)),
            ExprKind::Neg(a) => format!("(-({}))", self.ex(*a)),
            ExprKind::Ifx { c: cc, a, b } => {
                // MATLAB has no conditional expression: a branch that could fail when it is not the one taken (an index,
                // a call) is handed over unevaluated, so only the branch taken runs, as in the interpreter
                let lazy = self.may_fail(*a) || self.may_fail(*b);
                let cs = self.ex(*cc);
                let as_ = self.ex(*a);
                let bs = self.ex(*b);
                if lazy {
                    format!("{RTP}.choose_lazy({cs}, @() {as_}, @() {bs})")
                } else {
                    format!("{RTP}.choose({cs}, {as_}, {bs})")
                }
            }
            ExprKind::Bin { op, a, b } => self.bin(e, *op, *a, *b),
            ExprKind::Call { args, .. } => self.call(e, args),
        }
    }

    fn bin(&mut self, e: usize, op: BinOp, a: usize, b: usize) -> String {
        let ann = &self.b.c.ann[e];
        match op {
            BinOp::And => return format!("({} && {})", self.ex(a), self.ex(b)),
            BinOp::Or => return format!("({} || {})", self.ex(a), self.ex(b)),
            BinOp::Pow => return format!("{RTP}.ipow({}, {})", self.ex(a), jsfmt::num(ann.k)),
            BinOp::Ne => return format!("({} ~= {})", self.ex(a), self.ex(b)),
            _ => {}
        }
        if op.is_cmp() {
            return format!("({} {} {})", self.ex(a), op.text(), self.ex(b));
        }
        match ann.shape {
            Shape::Mv => {
                let (n, m) = match self.b.c.ann[a].ty.as_ref() {
                    Some(Ty::Arr(n, of)) => match &**of {
                        Ty::Arr(m, _) => (*n, *m),
                        _ => (*n, 0),
                    },
                    _ => (0, 0),
                };
                let small = |x: usize| (1..=4).contains(&x);
                let mat = if small(n) && small(m) { self.melems(a, n, m) } else { None };
                let v = if mat.is_some() { self.elems(b, m) } else { None };
                if let (Some(mb), Some(v)) = (mat, v) {
                    let rows: Vec<String> =
                        (1..=n).map(|i| (1..=m).map(|j| format!("{mb}({i}, {j})*{}", v[j - 1])).collect::<Vec<_>>().join(" + ")).collect();
                    return format!("[{}]", rows.join("; "));
                }
                format!("{RTP}.mv({}, {})", self.ex(a), self.ex(b))
            }
            Shape::Mm => format!("{RTP}.mm({}, {})", self.ex(a), self.ex(b)),
            _ => format!("({} {} {})", self.ex(a), op.text(), self.ex(b)),
        }
    }

    fn call(&mut self, e: usize, args: &[usize]) -> String {
        let c = self.b.c;
        let ann = &c.ann[e];
        if let Target::Record(ri) = ann.target {
            let r = &c.records[ri];
            return format!("{}.{}.{}_zero()", self.pkg, r.module, r.name);
        }
        let a: Vec<String> = args.iter().map(|&x| self.ex(x)).collect();
        let at = |i: usize| a.get(i).map_or("undefined", String::as_str);
        if let Target::Builtin(f) = ann.target {
            match f {
                "sin" | "cos" | "tan" | "atan" | "exp" | "erf" | "floor" | "ceil" | "round" => return format!("{f}({})", at(0)),
                // MATLAB's are complex outside the real domain (sqrt(-1), acos(1.5)): the runtime's give nan there, as the rest do
                "sqrt" | "asin" | "acos" | "log" | "log10" | "log2" => return format!("{RTP}.{f}_({})", at(0)),
                "trunc" => return format!("fix({})", at(0)),
                "abs" => return format!("{RTP}.fabs({})", at(0)),
                "sign" => return format!("sign({})", at(0)),
                "atan2" => return format!("atan2({}, {})", at(0), at(1)),
                "hypot" => return format!("hypot({}, {})", at(0), at(1)),
                "fmod" => return format!("rem({}, {})", at(0), at(1)),
                "pow" => return format!("{RTP}.pow_({}, {})", at(0), at(1)),
                "min" | "max" => {
                    let Some((first, rest)) = a.split_first() else {
                        return self.b.fail("Reduce of empty array with no initial value");
                    };
                    return rest.iter().fold(first.clone(), |acc, x| format!("{RTP}.f{f}({acc}, {x})"));
                }
                "clamp" => return format!("{RTP}.clamp({}, {}, {})", at(0), at(1), at(2)),
                "real" => return at(0).to_string(),
                "isnan" => return format!("isnan({})", at(0)),
                "isfinite" => return format!("isfinite({})", at(0)),
                "int" => return format!("fix({})", at(0)),
                "div" => return format!("fix(({}) / ({}))", at(0), at(1)),
                "rem" => return format!("rem({}, {})", at(0), at(1)),
                "band" => return format!("bitand({}, {})", at(0), at(1)),
                "bor" => return format!("bitor({}, {})", at(0), at(1)),
                "bxor" => return format!("bitxor({}, {})", at(0), at(1)),
                "shl" => return format!("bitshift({}, {})", at(0), at(1)),
                "shr" => return format!("bitshift({}, -({}))", at(0), at(1)),
                "len" => {
                    return match args.first().and_then(|&x| c.ann[x].ty.as_ref()) {
                        Some(Ty::Arr(n, _)) => n.to_string(),
                        Some(Ty::Buf(_)) => format!("numel({})", at(0)),
                        _ => "undefined".into(),
                    }
                }
                "dot" => {
                    let n = args.first().map_or(0, |&x| self.arr_len(x));
                    let aa = if (1..=4).contains(&n) { self.elems(args[0], n) } else { None };
                    let bb = if aa.is_some() { args.get(1).and_then(|&x| self.elems(x, n)) } else { None };
                    if let (Some(aa), Some(bb)) = (aa, bb) {
                        let t: Vec<String> = (0..n).map(|k| format!("{}*{}", aa[k], bb[k])).collect();
                        return format!("({})", t.join(" + "));
                    }
                    return format!("{RTP}.dot_({}, {})", at(0), at(1));
                }
                "cross" => {
                    let aa = args.first().and_then(|&x| self.elems(x, 3));
                    let bb = if aa.is_some() { args.get(1).and_then(|&x| self.elems(x, 3)) } else { None };
                    if let (Some(x), Some(y)) = (aa, bb) {
                        return format!(
                            "[{}*{} - {}*{}; {}*{} - {}*{}; {}*{} - {}*{}]",
                            x[1], y[2], x[2], y[1], x[2], y[0], x[0], y[2], x[0], y[1], x[1], y[0]
                        );
                    }
                    return format!("{RTP}.cross_({}, {})", at(0), at(1));
                }
                "norm" => {
                    let n = args.first().map_or(0, |&x| self.arr_len(x));
                    let aa = if (1..=4).contains(&n) { self.elems(args[0], n) } else { None };
                    if let Some(aa) = aa {
                        let t: Vec<String> = (0..n).map(|k| format!("{}*{}", aa[k], aa[k])).collect();
                        return format!("sqrt({})", t.join(" + "));
                    }
                    return format!("{RTP}.norm_({})", at(0));
                }
                "unit" => return format!("{RTP}.unit_({})", at(0)),
                "transpose" => return format!("({}).'", at(0)),
                "sort" => return format!("{RTP}.sort_({})", at(0)),
                "stream" => {
                    let id = match ann.sid {
                        Some((hi, lo)) => format!("[{}, {}]", jsfmt::num(hi), jsfmt::num(lo)),
                        None => at(1).to_string(),
                    };
                    return format!("{RTP}.stream_new({}, {id})", at(0));
                }
                "uniform" | "normal" | "normal3" => return format!("{RTP}.stream_{f}({})", at(0)),
                "argsort" => return format!("{RTP}.argsort_({})", at(0)),
                "eig" => return format!("{RTP}.eig_({})", at(0)),
                _ => {}
            }
        }
        let (module, name) = match ann.target {
            Target::Fn(fi) => (&c.fns[fi].module, &c.fns[fi].name),
            Target::Table(ti) => (&c.tables[ti].module, &c.tables[ti].name),
            _ => return self.b.fail("a call of nothing"),
        };
        format!("{}.{module}.{name}({})", self.pkg, a.join(", "))
    }

    fn lv(&mut self, l: usize) -> String {
        let c = self.b.c;
        match &c.exprs[l].kind {
            ExprKind::Var(name) => {
                if c.ann[l].var == VarKind::State {
                    format!("st.{name}")
                } else {
                    name.clone()
                }
            }
            ExprKind::Index { a, i } => {
                if let Some((aa, ai)) = self.matrix_pair(*a) {
                    let m = self.lv(aa);
                    let r = self.idx(ai);
                    return format!("{m}({r}, {})", self.idx(*i));
                }
                if self.rows_of(*a) {
                    let m = self.lv(*a);
                    return format!("{m}({}, :)", self.idx(*i));
                }
                let m = self.lv(*a);
                format!("{m}({})", self.idx(*i))
            }
            ExprKind::Field { a, f } => format!("{}.{f}", self.lv(*a)),
            _ => self.b.fail("not a place to set"),
        }
    }
    /// a matrix's row set: the column written is turned back into a row
    fn is_row_target(&mut self, l: usize) -> bool {
        let c = self.b.c;
        match &c.exprs[l].kind {
            ExprKind::Index { a, .. } => self.matrix_pair(*a).is_none() && self.rows_of(*a),
            _ => false,
        }
    }

    /// a call that changes its inout inputs: every output and each inout input's new value into a temporary, then
    /// the names or targets from the outputs, and the caller's variables from the inout values
    fn inout_stmt(&mut self, names: Option<&[String]>, targets: Option<&[usize]>, e: usize, ii: &str) -> String {
        let c = self.b.c;
        let ExprKind::Call { args, .. } = &c.exprs[e].kind else { return self.b.fail("an inout call of nothing") };
        let nouts = match c.ann[e].target {
            Target::Fn(fi) => c.fns[fi].outs.len(),
            Target::Builtin(_) => 1,
            _ => return self.b.fail("an inout call of nothing"),
        };
        let outs: Vec<String> = (0..nouts)
            .map(|_| {
                self.tmp += 1;
                format!("t__{}", self.tmp - 1)
            })
            .collect();
        let mut ios: Vec<(String, usize)> = Vec::new();
        for &i in &c.ann[e].inout_args {
            let Some(&a) = args.get(i) else { continue };
            ios.push((format!("t__{}", self.tmp), a));
            self.tmp += 1;
        }
        let all: Vec<&str> = outs.iter().map(String::as_str).chain(ios.iter().map(|x| x.0.as_str())).collect();
        let mut out = format!("{ii}[{}] = {};\n", all.join(", "), self.ex(e));
        if let Some(names) = names {
            for (n, t) in names.iter().zip(&outs) {
                out += &format!("{ii}{n} = {t};\n");
            }
        } else if let Some(targets) = targets {
            for (&tg, t) in targets.iter().zip(&outs) {
                let l = self.lv(tg);
                let tr = if self.is_row_target(tg) { ".'" } else { "" };
                out += &format!("{ii}{l} = {t}{tr};\n");
            }
        }
        for (tv, a) in ios {
            let l = self.lv(a);
            out += &format!("{ii}{l} = {tv};\n");
        }
        out
    }

    fn stmts(&mut self, body: &[Stmt], ind: usize) -> String {
        body.iter().map(|s| self.stmt(s, ind)).collect()
    }
    fn stmt(&mut self, s: &Stmt, ind: usize) -> String {
        let ii = "    ".repeat(ind);
        match &s.kind {
            StmtKind::Let { names, e, .. } => {
                if self.b.c.ann[*e].inout {
                    return self.inout_stmt(Some(names), None, *e, &ii);
                }
                if names.len() > 1 {
                    return format!("{ii}[{}] = {};\n", names.join(", "), self.ex(*e));
                }
                format!("{ii}{} = {};\n", names[0], self.ex(*e))
            }
            StmtKind::State { .. } => String::new(),
            StmtKind::Set { targets, e } => {
                if self.b.c.ann[*e].inout {
                    return self.inout_stmt(None, Some(targets), *e, &ii);
                }
                if targets.len() > 1 {
                    let ts: Vec<String> = targets
                        .iter()
                        .map(|_| {
                            self.tmp += 1;
                            format!("t__{}", self.tmp - 1)
                        })
                        .collect();
                    let mut out = format!("{ii}[{}] = {};\n", ts.join(", "), self.ex(*e));
                    for (t, n) in targets.iter().zip(&ts) {
                        let l = self.lv(*t);
                        let tr = if self.is_row_target(*t) { ".'" } else { "" };
                        out += &format!("{ii}{l} = {n}{tr};\n");
                    }
                    return out;
                }
                let t = targets[0];
                let l = self.lv(t);
                let v = self.ex(*e);
                let tr = if self.is_row_target(t) { ".'" } else { "" };
                format!("{ii}{l} = {v}{tr};\n")
            }
            StmtKind::If { arms, els } => {
                let mut out = String::new();
                for (i, (cond, body)) in arms.iter().enumerate() {
                    let cs = self.ex(*cond);
                    out += &format!("{ii}{} {cs}\n{}", if i > 0 { "elseif" } else { "if" }, self.stmts(body, ind + 1));
                }
                if let Some(els) = els {
                    out += &format!("{ii}else\n{}", self.stmts(els, ind + 1));
                }
                out + &format!("{ii}end\n")
            }
            StmtKind::For { v, a, b, body } => {
                if let Some(g) = self.guarded(v, body) {
                    let vn = format!("t__{}", self.tmp);
                    self.tmp += 1;
                    let as_ = self.ex(*a);
                    let bs = self.ex(*b);
                    let StmtKind::If { arms, .. } = &body[0].kind else { return String::new() };
                    let inner = self.stmts(&arms[0].1, ind + 1);
                    return format!("{ii}{vn} = (({as_}):(({bs}) - 1)).';\n{ii}for {v} = {vn}(logical({})).'\n{inner}{ii}end\n", g.replace(VMARK, &vn));
                }
                let as_ = self.ex(*a);
                let bs = self.ex(*b);
                format!("{ii}for {v} = ({as_}):(({bs}) - 1)\n{}{ii}end\n", self.stmts(body, ind + 1))
            }
            StmtKind::Settle { n: ne, c: ce, body, els } => {
                let k = format!("k__{}", self.tmp);
                self.tmp += 1;
                let n = format!("n__{}", self.tmp);
                self.tmp += 1;
                let ns = self.ex(*ne);
                let inner = self.stmts(body, ind + 1);
                let cs = self.ex(*ce);
                let es = match els {
                    Some(els) => self.stmts(els, ind + 2),
                    None => String::new(),
                };
                format!(
                    "{ii}{n} = {ns}; {k} = 0;\n{ii}while true\n{inner}{ii}    {k} = {k} + 1;\n{ii}    if {cs}, break; end\n{ii}    if {k} >= {n}\n{es}{ii}        break;\n{ii}    end\n{ii}end\n"
                )
            }
        }
    }

    fn run(&mut self) -> Files {
        let c = self.b.c;
        let head = head();
        let pkg = self.pkg.clone();
        let mut files = Files::new();
        for (mi, (mname, _)) in c.modules.iter().enumerate() {
            let dir = format!("+{mname}");
            for item in &c.mod_items[mi] {
                match *item {
                    ItemRef::Fn(fi) => {
                        let f = &c.fns[fi];
                        let mut outs: Vec<&str> = f.outs.iter().map(|o| o.name.as_str()).chain(f.params.iter().filter(|p| p.inout).map(|p| p.name.as_str())).collect();
                        let mut ins: Vec<&str> = f.params.iter().map(|p| p.name.as_str()).collect();
                        if f.kind == FnKind::Proc {
                            outs.push("st");
                            ins.insert(0, "st");
                        }
                        let mut body = String::new();
                        if f.kind == FnKind::Proc {
                            body += "    if isempty(st)\n        st = struct();\n";
                            for (name, e) in &f.states {
                                let v = self.b.const_of(*e);
                                let t = c.ann[*e].vty.as_ref().unwrap_or(&REAL);
                                body += &format!("        st.{name} = {};\n", self.value_lit_m(&v, t));
                            }
                            body += "    end\n";
                        }
                        for o in &f.outs {
                            body += &format!("    {} = {};\n", o.name, self.zero(&o.ty));
                        }
                        body += &self.stmts(&f.body, 1);
                        let mut lines: Vec<String> = f.doc.clone();
                        lines.extend(f.params.iter().map(io));
                        lines.extend(f.outs.iter().map(|o| format!("returns {}", io(o))));
                        lines.push(head.clone());
                        let text = format!("function [{}] = {}({})\n{}{body}end\n", outs.join(", "), f.name, ins.join(", "), help(&f.name, &lines));
                        set_file(&mut files, format!("{dir}/{}.m", f.name), text);
                    }
                    ItemRef::Table(ti) => {
                        let t = &c.tables[ti];
                        let outs: Vec<&str> = t.outs.iter().map(|o| o.name.as_str()).collect();
                        let key = &t.key.name;
                        let mode = mode_text(t.mode);
                        let mut lines: Vec<String> = t.doc.clone();
                        lines.push(format!("table ({mode}): {key} -> {}", outs.join(", ")));
                        lines.push(head.clone());
                        let rows: Vec<String> = t
                            .si
                            .iter()
                            .map(|r| {
                                let xs: Vec<String> = r.iter().map(|x| ml(&mut self.b, *x)).collect();
                                format!("        {}", xs.join(", "))
                            })
                            .collect();
                        let gives: String = outs.iter().enumerate().map(|(i, o)| format!("    {o} = r({});\n", i + 1)).collect();
                        let text = format!(
                            "function [{}] = {}({key})\n{}    T = [\n{}\n    ];\n    r = {RTP}.lookup_{mode}(T, {key});\n{gives}end\n",
                            outs.join(", "),
                            t.name,
                            help(&t.name, &lines),
                            rows.join("\n")
                        );
                        set_file(&mut files, format!("{dir}/{}.m", t.name), text);
                    }
                    ItemRef::Data(di) => {
                        // one copy, kept between calls: v = NAME() is the whole table, NAME(i, j) an element, NAME(i, ':') a row
                        let d = &c.data[di];
                        let (rows, vals, el) = data_rows(d);
                        let m = |b: &mut Base, x: f64| if el == Ty::Int { jsfmt::num(x) } else { ml(b, x) };
                        let lines: Vec<String> = match rows {
                            Some(rows) => rows.iter().map(|r| format!("            {}", r.iter().map(|x| m(&mut self.b, *x)).collect::<Vec<_>>().join(", "))).collect(),
                            None => vals.chunks(10).map(|ch| format!("            {}", ch.iter().map(|x| m(&mut self.b, *x)).collect::<Vec<_>>().join("; "))).collect(),
                        };
                        let mut hl: Vec<String> = d.doc.clone();
                        hl.push(format!("data: {}; SI: {}", decl_text(&d.decl), si_of(&d.ty)));
                        hl.push(head.clone());
                        let text = format!(
                            "function v = {0}(varargin)\n{1}    persistent T\n    if isempty(T)\n        T = [\n{2}\n        ];\n    end\n    if nargin == 0, v = T; else, v = T(varargin{{:}}); end\nend\n",
                            d.name,
                            help(&d.name, &hl),
                            lines.join("\n")
                        );
                        set_file(&mut files, format!("{dir}/{}.m", d.name), text);
                    }
                    ItemRef::Record(ri) => {
                        let r = &c.records[ri];
                        let fields: String = r.fields.iter().map(|f| format!("    s.{} = {};\n", f.name, self.zero(&f.ty))).collect();
                        let text = format!(
                            "function s = {0}_zero()\n{1}    s = struct();\n{fields}end\n",
                            r.name,
                            help(&format!("{}_zero", r.name), &[format!("a {} with every field zero", r.name), head.clone()])
                        );
                        set_file(&mut files, format!("{dir}/{}_zero.m", r.name), text);
                    }
                    ItemRef::Const(_) | ItemRef::Choice(_) => {}
                }
            }
        }
        // the vector dispatcher: a function by name, its inputs and outputs flattened (row-major, SI); a record field by
        // field in declaration order, as the vectors flatten it
        let mut disp = format!(
            "function y = call(name, x)\n{}    switch name\n",
            help("call", &["a function by its registry name (module::name), inputs and outputs flattened (row-major, SI)".into(), head.clone()])
        );
        for f in c.fns.iter().filter(|f| f.kind == FnKind::Fn) {
            if f.params.iter().any(|p| matches!(p.ty, Ty::Buf(_))) {
                disp += &self.buf_call(f, &pkg);
                continue;
            }
            let mut at = 0;
            let mut args = Vec::new();
            for p in &f.params {
                args.push(self.read_arg(&p.ty, at, "x"));
                at += self.b.flat(&p.ty);
            }
            let outs: Vec<String> = (0..f.outs.len()).map(|i| format!("o{}", i + 1)).chain(io_outs(f).into_iter().map(|x| x.0)).collect();
            let ys: Vec<String> = f.outs.iter().map(|o| &o.ty).chain(io_outs(f).into_iter().map(|x| x.1)).zip(&outs).map(|(t, v)| self.flat_out(t, v)).collect();
            disp += &format!(
                "        case '{0}::{1}'\n            [{2}] = {pkg}.{0}.{1}({3});\n            y = [{4}];\n",
                f.module,
                f.name,
                outs.join(", "),
                args.join(", "),
                ys.join("; ")
            );
        }
        disp += "        otherwise\n            error('pcode:call', 'no function %s', name);\n    end\nend\n";
        set_file(&mut files, "call.m".into(), disp);
        let mut seq = format!(
            "function Y = call_seq(name, X)\n{}    st = []; Y = [];\n    for c = 1:size(X, 2)\n        x = X(:, c);\n        switch name\n",
            help("call_seq", &["a proc called once per column of X, its state carried from each call to the next; Y has a column per call".into(), head.clone()])
        );
        for f in c.fns.iter().filter(|f| f.kind == FnKind::Proc) {
            let mut at = 0;
            let mut args = Vec::new();
            for p in &f.params {
                args.push(self.read_arg(&p.ty, at, "x"));
                at += self.b.flat(&p.ty);
            }
            let outs: Vec<String> = (0..f.outs.len()).map(|i| format!("o{}", i + 1)).chain(io_outs(f).into_iter().map(|x| x.0)).collect();
            let ys: Vec<String> = f.outs.iter().map(|o| &o.ty).chain(io_outs(f).into_iter().map(|x| x.1)).zip(&outs).map(|(t, v)| self.flat_out(t, v)).collect();
            seq += &format!(
                "            case '{0}::{1}'\n                [{2}, st] = {pkg}.{0}.{1}(st{3}{4});\n                y = [{5}];\n",
                f.module,
                f.name,
                outs.join(", "),
                if args.is_empty() { "" } else { ", " },
                args.join(", "),
                ys.join("; ")
            );
        }
        seq += "            otherwise\n                error('pcode:call', 'no proc %s', name);\n        end\n        Y = [Y, y];\n    end\nend\n";
        set_file(&mut files, "call_seq.m".into(), seq);
        files
    }

    fn read_arg(&self, t: &Ty, at: usize, x: &str) -> String {
        let n = self.b.flat(t);
        if let Ty::Rec(name) = t {
            let mut k = 0;
            let parts: Vec<String> = match self.b.record(name) {
                Some(r) => r
                    .fields
                    .iter()
                    .map(|f| {
                        let s = format!("'{}', {{{}}}", f.name, self.read_arg(&f.ty, at + k, x));
                        k += self.b.flat(&f.ty);
                        s
                    })
                    .collect(),
                None => Vec::new(),
            };
            return format!("struct({})", parts.join(", "));
        }
        let sl = if n == 1 { format!("{x}({})", at + 1) } else { format!("{x}({}:{})", at + 1, at + n) };
        match t {
            Ty::Bool => format!("({sl} ~= 0)"),
            Ty::Stream => format!("reshape({sl}, 6, 1)"),
            Ty::Arr(n, of) => match &**of {
                Ty::Arr(m, _) => format!("reshape({sl}, {m}, {n}).'"),
                _ => format!("reshape({sl}, {n}, 1)"),
            },
            _ => sl,
        }
    }
    /// The dispatcher's case of a function with a buffer (its length the caller's): the inputs read in turn, a buffer
    /// its length first.
    fn buf_call(&self, f: &crate::ast::Func, pkg: &str) -> String {
        let ii = "            ";
        let mut s = format!("        case '{}::{}'\n{ii}at = 0;\n", f.module, f.name);
        let mut args = Vec::new();
        for (i, p) in f.params.iter().enumerate() {
            let k = i + 1;
            if matches!(p.ty, Ty::Buf(_)) {
                s += &format!("{ii}n{k} = x(at + 1);\n{ii}a{k} = reshape(x(at + 2:at + 1 + n{k}), [], 1);\n{ii}at = at + 1 + n{k};\n");
            } else {
                s += &format!("{ii}xa = x(at + 1:end);\n{ii}a{k} = {};\n{ii}at = at + {};\n", self.read_arg(&p.ty, 0, "xa"), self.b.flat(&p.ty));
            }
            args.push(format!("a{k}"));
        }
        let outs: Vec<String> = (0..f.outs.len()).map(|i| format!("o{}", i + 1)).collect();
        let names: Vec<String> = outs.iter().cloned().chain(io_outs(f).into_iter().map(|x| x.0)).collect();
        let ys: Vec<String> = f
            .outs
            .iter()
            .zip(&outs)
            .map(|(o, v)| self.flat_out(&o.ty, v))
            .chain(io_outs(f).into_iter().map(|(n, t)| if matches!(t, Ty::Buf(_)) { format!("numel({n}); reshape({n}, [], 1)") } else { self.flat_out(t, &n) }))
            .collect();
        s + &format!("{ii}[{}] = {pkg}.{}.{}({});\n{ii}y = [{}];\n", names.join(", "), f.module, f.name, args.join(", "), ys.join("; "))
    }
    fn flat_out(&self, t: &Ty, v: &str) -> String {
        match t {
            Ty::Rec(name) => {
                let parts: Vec<String> = match self.b.record(name) {
                    Some(r) => r.fields.iter().map(|f| self.flat_out(&f.ty, &format!("{v}.{}", f.name))).collect(),
                    None => Vec::new(),
                };
                format!("[{}]", parts.join("; "))
            }
            Ty::Arr(_, of) if matches!(**of, Ty::Arr(..)) => format!("reshape(({v}).', [], 1)"),
            Ty::Arr(..) | Ty::Stream => format!("reshape({v}, [], 1)"),
            _ => format!("double({v})"),
        }
    }
}

/// an inout input's value after the call, a dispatcher's output after the outputs: its name and type
fn io_outs(f: &crate::ast::Func) -> Vec<(String, &Ty)> {
    f.params.iter().enumerate().filter(|(_, p)| p.inout).map(|(i, p)| (format!("io{}", i + 1), &p.ty)).collect()
}

/// the shared MATLAB runtime (+asils/+pc): written once, the same arithmetic as the interpreter
pub(super) fn runtime() -> Files {
    let head = head();
    let mut f = Files::new();
    let mut add = |name: &str, args: &str, outs: &str, doc: &str, body: &str| {
        set_file(&mut f, format!("{name}.m"), format!("function {outs} = {name}({args})\n%{}  {doc}\n%   {head}\n{body}end\n", upper(name)));
    };
    add(
        "ipow",
        "x, k",
        "r",
        "x^k by repeated multiplication, left to right (x^-k = 1/(x^k)).",
        "    if k == 0, r = 1; return; end\n    r = x;\n    for i = 2:abs(k), r = r*x; end\n    if k < 0, r = 1/r; end\n",
    );
    add("fmin", "a, b", "r", "the smaller (b only when b < a).", "    if b < a, r = b; else, r = a; end\n");
    add("fmax", "a, b", "r", "the larger (b only when b > a).", "    if b > a, r = b; else, r = a; end\n");
    add("fabs", "x", "r", "|x|.", "    if x < 0, r = -x; elseif x == 0, r = 0; else, r = x; end\n");
    add("clamp", "x, lo, hi", "r", "min(max(x, lo), hi).", "    r = asils.pc.fmin(asils.pc.fmax(x, lo), hi);\n");
    add("choose", "c, a, b", "r", "a when c, else b (both are evaluated).", "    if c, r = a; else, r = b; end\n");
    add("choose_lazy", "c, a, b", "r", "a() when c, else b(): only the branch taken is evaluated.", "    if c, r = a(); else, r = b(); end\n");
    add("dot_", "a, b", "s", "sum of products, left to right.", "    if numel(a) == 3\n        s = a(1)*b(1) + a(2)*b(2) + a(3)*b(3);\n    else\n        s = a(1)*b(1);\n        for i = 2:numel(a), s = s + a(i)*b(i); end\n    end\n");
    add("cross_", "a, b", "c", "cross product of two 3-vectors.", "    c = [a(2)*b(3) - a(3)*b(2); a(3)*b(1) - a(1)*b(3); a(1)*b(2) - a(2)*b(1)];\n");
    add("norm_", "a", "n", "sqrt(dot(a, a)).", "    n = sqrt(asils.pc.dot_(a, a));\n");
    add("unit_", "a", "u", "a / max(norm(a), 1e-30).", "    u = a / asils.pc.fmax(asils.pc.norm_(a), 1e-30);\n");
    add(
        "mv",
        "M, v",
        "r",
        "matrix times vector, each row summed left to right.",
        "    r = zeros(size(M, 1), 1);\n    for i = 1:size(M, 1), r(i) = asils.pc.dot_(M(i, :), v); end\n",
    );
    add(
        "mm",
        "A, B",
        "C",
        "matrix product, summed left to right.",
        "    C = zeros(size(A, 1), size(B, 2));\n    for i = 1:size(A, 1)\n        for j = 1:size(B, 2)\n            s = A(i, 1)*B(1, j);\n            for k = 2:size(B, 1), s = s + A(i, k)*B(k, j); end\n            C(i, j) = s;\n        end\n    end\n",
    );
    add(
        "lookup_step",
        "T, x",
        "r",
        "the last row whose key is at or below x (the first row below the first key).",
        "    i = 1;\n    while i + 1 <= size(T, 1) && T(i + 1, 1) <= x, i = i + 1; end\n    r = T(i, :);\n",
    );
    add(
        "lookup_linear",
        "T, x",
        "r",
        "straight-line between the two rows around x, held at the ends.",
        "    n = size(T, 1);\n    xc = asils.pc.clamp(x, T(1, 1), T(n, 1));\n    i = 1;\n    while i + 2 <= n && T(i + 1, 1) <= xc, i = i + 1; end\n    s = (xc - T(i, 1))/(T(i + 1, 1) - T(i, 1));\n    r = T(i, :);\n    for j = 1:size(T, 2), r(j) = T(i, j) + s*(T(i + 1, j) - T(i, j)); end\n",
    );
    add(
        "argsort_",
        "v",
        "ix",
        "the indices (from 0) of a vector in ascending order, equal values in their order: an insertion sort.",
        "    ix = (0:numel(v) - 1).';\n    for i = 2:numel(ix)\n        k = ix(i); j = i - 1;\n        while j >= 1 && v(ix(j) + 1) > v(k + 1), ix(j + 1) = ix(j); j = j - 1; end\n        ix(j + 1) = k;\n    end\n",
    );
    add("sort_", "v", "r", "the vector in ascending order, equal values in their order.", "    r = reshape(v(asils.pc.argsort_(v) + 1), [], 1);\n");
    // the functions whose MATLAB value is complex outside their real domain: nan there, as every other translation gives
    add("sqrt_", "x", "r", "sqrt(x), nan where x < 0 (MATLAB's is complex there; C's, Rust's and the interpreter's nan).", "    r = sqrt(x);\n    if ~isreal(r), r = NaN; end\n");
    add("asin_", "x", "r", "asin(x), nan where |x| > 1 (MATLAB's is complex there; C's, Rust's and the interpreter's nan).", "    r = asin(x);\n    if ~isreal(r), r = NaN; end\n");
    add("acos_", "x", "r", "acos(x), nan where |x| > 1 (MATLAB's is complex there; C's, Rust's and the interpreter's nan).", "    r = acos(x);\n    if ~isreal(r), r = NaN; end\n");
    add("log_", "x", "r", "log(x), nan where x < 0 (MATLAB's is complex there; C's, Rust's and the interpreter's nan).", "    r = log(x);\n    if ~isreal(r), r = NaN; end\n");
    add("log10_", "x", "r", "log10(x), nan where x < 0 (MATLAB's is complex there; C's, Rust's and the interpreter's nan).", "    r = log10(x);\n    if ~isreal(r), r = NaN; end\n");
    add("log2_", "x", "r", "log2(x), nan where x < 0 (MATLAB's is complex there; C's, Rust's and the interpreter's nan).", "    r = log2(x);\n    if ~isreal(r), r = NaN; end\n");
    add("pow_", "x, y", "r", "x^y, nan where x < 0 and y is not whole (MATLAB's is complex there; C's, Rust's and the interpreter's nan).", "    r = x^y;\n    if ~isreal(r), r = NaN; end\n");
    // the toolbox's eigenvalues (pcode.js rt.eig): the same arithmetic, element by element
    add("eig_sgn_", "x, y", "r", "|x| with the sign of y (y = 0 or -0: positive).", "    if y >= 0, r = abs(x); else, r = -abs(x); end\n");
    add(
        "eig_",
        "M",
        "e",
        "the eigenvalues of a real square matrix, a row [re, im] each, in the order the QR iteration leaves them (pcode.js rt.eig): EISPACK's balanc by powers of 2, orthes and hqr.",
        r#"    a = M; n = size(a, 1); wr = zeros(n, 1); wi = zeros(n, 1); ort = zeros(n, 1);
    done = false; sweep = 0;
    while sweep < 100 && ~done
        done = true;
        for i = 0:n - 1
            c = 0; r = 0;
            for j = 0:n - 1
                if j ~= i, c = c + abs(a(j + 1, i + 1)); r = r + abs(a(i + 1, j + 1)); end
            end
            if c ~= 0 && r ~= 0 && isfinite(c + r)
                g = r / 2; f = 1; s = c + r;
                while c < g, f = f * 2; c = c * 4; end
                g = r * 2;
                while c > g, f = f / 2; c = c / 4; end
                if (c + r) / f < 0.95 * s
                    done = false;
                    g = 1 / f;
                    for j = 0:n - 1, a(i + 1, j + 1) = a(i + 1, j + 1) * g; end
                    for j = 0:n - 1, a(j + 1, i + 1) = a(j + 1, i + 1) * f; end
                end
            end
        end
        sweep = sweep + 1;
    end
    for m = 1:n - 2
        h = 0; scale = 0; ort(m + 1) = 0;
        for i = m:n - 1, scale = scale + abs(a(i + 1, m)); end
        if scale ~= 0
            for i = n - 1:-1:m, ort(i + 1) = a(i + 1, m) / scale; h = h + ort(i + 1) * ort(i + 1); end
            g = -asils.pc.eig_sgn_(sqrt(h), ort(m + 1));
            h = h - ort(m + 1) * g;
            ort(m + 1) = ort(m + 1) - g;
            for j = m:n - 1
                f = 0;
                for i = n - 1:-1:m, f = f + ort(i + 1) * a(i + 1, j + 1); end
                f = f / h;
                for i = m:n - 1, a(i + 1, j + 1) = a(i + 1, j + 1) - f * ort(i + 1); end
            end
            for i = 0:n - 1
                f = 0;
                for j = n - 1:-1:m, f = f + ort(j + 1) * a(i + 1, j + 1); end
                f = f / h;
                for j = m:n - 1, a(i + 1, j + 1) = a(i + 1, j + 1) - f * ort(j + 1); end
            end
            ort(m + 1) = scale * ort(m + 1);
            a(m + 1, m) = scale * g;
        end
    end
    for i = 2:n - 1
        for j = 0:i - 2, a(i + 1, j + 1) = 0; end
    end
    nrm = 0; k = 0;
    for i = 0:n - 1
        for j = k:n - 1, nrm = nrm + abs(a(i + 1, j + 1)); end
        k = i;
    end
    left = n; t = 0; itn = 30 * n;
    while left > 0
        en = left - 1; its = 0;
        while true
            l = en;
            while l > 0
                s = abs(a(l, l)) + abs(a(l + 1, l + 1));
                if s == 0, s = nrm; end
                if s + abs(a(l + 1, l)) == s, break; end
                l = l - 1;
            end
            x = a(en + 1, en + 1);
            if l == en, wr(en + 1) = x + t; wi(en + 1) = 0; left = left - 1; break; end
            na = en - 1;
            y = a(na + 1, na + 1);
            w = a(en + 1, na + 1) * a(na + 1, en + 1);
            if l == na
                p = (y - x) / 2;
                q = p * p + w;
                zz = sqrt(abs(q));
                x = x + t;
                if q >= 0
                    zz = p + asils.pc.eig_sgn_(zz, p);
                    wr(na + 1) = x + zz; wr(en + 1) = wr(na + 1);
                    if zz ~= 0, wr(en + 1) = x - w / zz; end
                    wi(na + 1) = 0; wi(en + 1) = 0;
                else
                    wr(na + 1) = x + p; wr(en + 1) = x + p; wi(na + 1) = zz; wi(en + 1) = -zz;
                end
                left = left - 2;
                break;
            end
            if itn == 0, wr(1:en + 1) = NaN; wi(1:en + 1) = NaN; left = 0; break; end
            enm2 = na - 1;
            if its == 10 || its == 20
                t = t + x;
                for i = 0:en, a(i + 1, i + 1) = a(i + 1, i + 1) - x; end
                s = abs(a(en + 1, na + 1)) + abs(a(na + 1, enm2 + 1));
                x = 0.75 * s; y = x; w = -0.4375 * s * s;
            end
            its = its + 1; itn = itn - 1;
            m = enm2;
            while true
                zz = a(m + 1, m + 1);
                r = x - zz;
                s = y - zz;
                p = (r * s - w) / a(m + 2, m + 1) + a(m + 1, m + 2);
                q = a(m + 2, m + 2) - zz - r - s;
                r = a(m + 3, m + 2);
                s = abs(p) + abs(q) + abs(r);
                p = p / s; q = q / s; r = r / s;
                if m == l, break; end
                tst1 = abs(p) * (abs(a(m, m)) + abs(zz) + abs(a(m + 2, m + 2)));
                if tst1 + abs(a(m + 1, m)) * (abs(q) + abs(r)) == tst1, break; end
                m = m - 1;
            end
            for i = m + 2:en
                a(i + 1, i - 1) = 0;
                if i ~= m + 2, a(i + 1, i - 2) = 0; end
            end
            for k = m:na
                notlas = k ~= na;
                if k ~= m
                    p = a(k + 1, k); q = a(k + 2, k); r = 0;
                    if notlas, r = a(k + 3, k); end
                    x = abs(p) + abs(q) + abs(r);
                    if x == 0, continue; end
                    p = p / x; q = q / x; r = r / x;
                end
                s = asils.pc.eig_sgn_(sqrt(p * p + q * q + r * r), p);
                if k ~= m
                    a(k + 1, k) = -s * x;
                elseif l ~= m
                    a(k + 1, k) = -a(k + 1, k);
                end
                p = p + s; x = p / s; y = q / s; zz = r / s; q = q / p; r = r / p;
                if k + 3 < en, jm = k + 3; else, jm = en; end
                if notlas
                    for j = k:en
                        p = a(k + 1, j + 1) + q * a(k + 2, j + 1) + r * a(k + 3, j + 1);
                        a(k + 1, j + 1) = a(k + 1, j + 1) - p * x; a(k + 2, j + 1) = a(k + 2, j + 1) - p * y; a(k + 3, j + 1) = a(k + 3, j + 1) - p * zz;
                    end
                    for i = l:jm
                        p = x * a(i + 1, k + 1) + y * a(i + 1, k + 2) + zz * a(i + 1, k + 3);
                        a(i + 1, k + 1) = a(i + 1, k + 1) - p; a(i + 1, k + 2) = a(i + 1, k + 2) - p * q; a(i + 1, k + 3) = a(i + 1, k + 3) - p * r;
                    end
                else
                    for j = k:en
                        p = a(k + 1, j + 1) + q * a(k + 2, j + 1);
                        a(k + 1, j + 1) = a(k + 1, j + 1) - p * x; a(k + 2, j + 1) = a(k + 2, j + 1) - p * y;
                    end
                    for i = l:jm
                        p = x * a(i + 1, k + 1) + y * a(i + 1, k + 2);
                        a(i + 1, k + 1) = a(i + 1, k + 1) - p; a(i + 1, k + 2) = a(i + 1, k + 2) - p * q;
                    end
                end
            end
        end
    end
    e = [wr, wi];
"#,
    );
    // random streams (pcode.js rt.stream: adcs-sim-core rng.rs): a stream is six numbers, its 64-bit key and counter as
    // 32-bit halves; the 64-bit arithmetic in halves and 16-bit limbs, every step exact in doubles
    add(
        "u64_of_",
        "x",
        "h",
        "a whole number (below 2^53 in size) as the 32-bit halves [high, low] of its 64-bit two's complement.",
        "    if x >= 0\n        h = [floor(x / 4294967296), mod(x, 4294967296)];\n    else\n        a = -x; lo = 4294967296 - mod(a, 4294967296); hi = 4294967295 - floor(a / 4294967296);\n        if lo == 4294967296, lo = 0; hi = hi + 1; end\n        h = [mod(hi, 4294967296), lo];\n    end\n",
    );
    add(
        "u64_add_",
        "a, b",
        "r",
        "a + b modulo 2^64, in 32-bit halves [high, low].",
        "    lo = a(2) + b(2); c = floor(lo / 4294967296); lo = lo - c*4294967296;\n    r = [mod(a(1) + b(1) + c, 4294967296), lo];\n",
    );
    add("u64_xor_", "a, b", "r", "a xor b, in 32-bit halves [high, low].", "    r = [bitxor(a(1), b(1)), bitxor(a(2), b(2))];\n");
    add(
        "u64_shr_",
        "a, k",
        "r",
        "a shifted right by k (0 < k < 32), in 32-bit halves [high, low].",
        "    r = [floor(a(1) / 2^k), mod(a(1), 2^k)*2^(32 - k) + floor(a(2) / 2^k)];\n",
    );
    add(
        "u64_mul_",
        "a, b",
        "r",
        "a * b modulo 2^64, in 32-bit halves [high, low]: by 16-bit limbs, each sum exact.",
        "    x = [mod(a(2), 65536), floor(a(2) / 65536), mod(a(1), 65536), floor(a(1) / 65536)];\n    y = [mod(b(2), 65536), floor(b(2) / 65536), mod(b(1), 65536), floor(b(1) / 65536)];\n    c0 = x(1)*y(1);\n    c1 = x(1)*y(2) + x(2)*y(1);\n    c2 = x(1)*y(3) + x(2)*y(2) + x(3)*y(1);\n    c3 = x(1)*y(4) + x(2)*y(3) + x(3)*y(2) + x(4)*y(1);\n    r0 = mod(c0, 65536); t = floor(c0 / 65536) + c1;\n    r1 = mod(t, 65536); t = floor(t / 65536) + c2;\n    r2 = mod(t, 65536); t = floor(t / 65536) + c3;\n    r = [r2 + mod(t, 65536)*65536, r0 + r1*65536];\n",
    );
    add(
        "sm64_",
        "z",
        "z",
        "SplitMix64 of a 64-bit number in 32-bit halves [high, low] (adcs-sim-core rng.rs splitmix64).",
        "    z = asils.pc.u64_add_(z, [2654435769, 2135587861]);\n    z = asils.pc.u64_mul_(asils.pc.u64_xor_(z, asils.pc.u64_shr_(z, 30)), [3210233709, 484763065]);\n    z = asils.pc.u64_mul_(asils.pc.u64_xor_(z, asils.pc.u64_shr_(z, 27)), [2496678331, 321982955]);\n    z = asils.pc.u64_xor_(z, asils.pc.u64_shr_(z, 31));\n",
    );
    add(
        "stream_new",
        "seed, id",
        "s",
        "the random stream of a seed and an id (a whole number, or a name's [high, low] halves).",
        "    if numel(id) == 2, i = reshape(id, 1, 2); else, i = asils.pc.u64_of_(id); end\n    k = asils.pc.sm64_(asils.pc.u64_xor_(asils.pc.u64_of_(seed), asils.pc.sm64_(i)));\n    s = [k(1); k(2); 0; 0; 0; 0];\n",
    );
    add(
        "stream_block",
        "kh, kl, nh, nl",
        "w",
        "the 257 uniform draws of the stream with key [kh, kl] at the counters [nh, nl] + 1 on, as stream_uniform draws them.",
        "    % the counters [nh, nl] + 1 to + 257, each its own SplitMix64 as stream_uniform makes it, element by element\n    zl = nl + (1:257)'; zh = nh + zeros(257, 1);\n    c = zl >= 4294967296; zl(c) = zl(c) - 4294967296; zh(c) = mod(zh(c) + 1, 4294967296);\n    lo = zl + 2135587861; cy = floor(lo / 4294967296); zl = lo - cy*4294967296; zh = mod(zh + 2654435769 + cy, 4294967296);\n    zh_ = floor(zh / 1073741824); zl_ = mod(zh, 1073741824)*4 + floor(zl / 1073741824); zh = bitxor(zh, zh_); zl = bitxor(zl, zl_);\n    x1 = mod(zl, 65536); x2 = floor(zl / 65536); x3 = mod(zh, 65536); x4 = floor(zh / 65536);\n    c0 = x1*58809; c1 = x1*7396 + x2*58809; c2 = x1*18285 + x2*7396 + x3*58809; c3 = x1*48984 + x2*18285 + x3*7396 + x4*58809;\n    r0 = mod(c0, 65536); t = floor(c0 / 65536) + c1; r1 = mod(t, 65536); t = floor(t / 65536) + c2;\n    r2 = mod(t, 65536); t = floor(t / 65536) + c3; zh = r2 + mod(t, 65536)*65536; zl = r0 + r1*65536;\n    zh_ = floor(zh / 134217728); zl_ = mod(zh, 134217728)*32 + floor(zl / 134217728); zh = bitxor(zh, zh_); zl = bitxor(zl, zl_);\n    x1 = mod(zl, 65536); x2 = floor(zl / 65536); x3 = mod(zh, 65536); x4 = floor(zh / 65536);\n    c0 = x1*4587; c1 = x1*4913 + x2*4587; c2 = x1*18875 + x2*4913 + x3*4587; c3 = x1*38096 + x2*18875 + x3*4913 + x4*4587;\n    r0 = mod(c0, 65536); t = floor(c0 / 65536) + c1; r1 = mod(t, 65536); t = floor(t / 65536) + c2;\n    r2 = mod(t, 65536); t = floor(t / 65536) + c3; zh = r2 + mod(t, 65536)*65536; zl = r0 + r1*65536;\n    zh_ = floor(zh / 2147483648); zl_ = mod(zh, 2147483648)*2 + floor(zl / 2147483648); zh = bitxor(zh, zh_); zl = bitxor(zl, zl_);\n    zh = bitxor(kh, zh); zl = bitxor(kl, zl);\n    lo = zl + 2135587861; cy = floor(lo / 4294967296); zl = lo - cy*4294967296; zh = mod(zh + 2654435769 + cy, 4294967296);\n    zh_ = floor(zh / 1073741824); zl_ = mod(zh, 1073741824)*4 + floor(zl / 1073741824); zh = bitxor(zh, zh_); zl = bitxor(zl, zl_);\n    x1 = mod(zl, 65536); x2 = floor(zl / 65536); x3 = mod(zh, 65536); x4 = floor(zh / 65536);\n    c0 = x1*58809; c1 = x1*7396 + x2*58809; c2 = x1*18285 + x2*7396 + x3*58809; c3 = x1*48984 + x2*18285 + x3*7396 + x4*58809;\n    r0 = mod(c0, 65536); t = floor(c0 / 65536) + c1; r1 = mod(t, 65536); t = floor(t / 65536) + c2;\n    r2 = mod(t, 65536); t = floor(t / 65536) + c3; zh = r2 + mod(t, 65536)*65536; zl = r0 + r1*65536;\n    zh_ = floor(zh / 134217728); zl_ = mod(zh, 134217728)*32 + floor(zl / 134217728); zh = bitxor(zh, zh_); zl = bitxor(zl, zl_);\n    x1 = mod(zl, 65536); x2 = floor(zl / 65536); x3 = mod(zh, 65536); x4 = floor(zh / 65536);\n    c0 = x1*4587; c1 = x1*4913 + x2*4587; c2 = x1*18875 + x2*4913 + x3*4587; c3 = x1*38096 + x2*18875 + x3*4913 + x4*4587;\n    r0 = mod(c0, 65536); t = floor(c0 / 65536) + c1; r1 = mod(t, 65536); t = floor(t / 65536) + c2;\n    r2 = mod(t, 65536); t = floor(t / 65536) + c3; zh = r2 + mod(t, 65536)*65536; zl = r0 + r1*65536;\n    zh_ = floor(zh / 2147483648); zl_ = mod(zh, 2147483648)*2 + floor(zl / 2147483648); zh = bitxor(zh, zh_); zl = bitxor(zl, zl_);\n    w = (zh*2097152 + floor(zl / 2048) + 0.5)*(1/9007199254740992);\n",
    );
    add(
        "stream_uniform",
        "s",
        "[u, s]",
        "a uniform draw on (0, 1), and the stream advanced.",
        "    % a draw is a function of the stream's key and counter alone: the next 257 draws of a key are made at once\n    % (stream_block) and kept, and a run of draws reads them (the same numbers, sooner)\n    persistent K1 K2 BH BL U r\n    if isempty(r), K1 = NaN(64, 1); K2 = K1; BH = K1; BL = K1; U = zeros(64, 257); r = 1; end\n    nl = s(4) + 1; nh = s(3);\n    if nl == 4294967296, nl = 0; nh = mod(nh + 1, 4294967296); end\n    if ~(K1(r) == s(1) && K2(r) == s(2))\n        r = find(K1 == s(1) & K2 == s(2), 1);\n        if isempty(r)\n            r = find(isnan(K1), 1); if isempty(r), r = 1 + mod(nl, 64); end\n            K1(r) = s(1); K2(r) = s(2); BH(r) = NaN;\n        end\n    end\n    j = nl - BL(r);\n    if ~(BH(r) == nh && j >= 1 && j <= 256)\n        BH(r) = nh; BL(r) = nl - 1; U(r, :) = asils.pc.stream_block(s(1), s(2), nh, nl - 1).'; j = 1;\n    end\n    u = U(r, j);\n    s(3) = nh; s(4) = nl;\n",
    );
    add(
        "stream_normal",
        "s",
        "[z, s]",
        "a normal draw by Box-Muller (the spare kept), and the stream advanced.",
        "    persistent K1 K2 BH BL U r\n    if s(6) ~= 0\n        z = s(5); s(5) = 0; s(6) = 0;\n        return;\n    end\n    % its two uniform draws read from kept blocks of stream_block, as stream_uniform reads them\n    if isempty(r), K1 = NaN(64, 1); K2 = K1; BH = K1; BL = K1; U = zeros(64, 257); r = 1; end\n    nl = s(4) + 1; nh = s(3);\n    if nl == 4294967296, nl = 0; nh = mod(nh + 1, 4294967296); end\n    if ~(K1(r) == s(1) && K2(r) == s(2))\n        r = find(K1 == s(1) & K2 == s(2), 1);\n        if isempty(r)\n            r = find(isnan(K1), 1); if isempty(r), r = 1 + mod(nl, 64); end\n            K1(r) = s(1); K2(r) = s(2); BH(r) = NaN;\n        end\n    end\n    j = nl - BL(r);\n    if ~(BH(r) == nh && j >= 1 && j <= 256)\n        BH(r) = nh; BL(r) = nl - 1; U(r, :) = asils.pc.stream_block(s(1), s(2), nh, nl - 1).'; j = 1;\n    end\n    u1 = U(r, j); u2 = U(r, j + 1);\n    if nl == 4294967295, s(3) = mod(nh + 1, 4294967296); s(4) = 0; else, s(3) = nh; s(4) = nl + 1; end\n    rr = sqrt(-2*log(u1));\n    s(5) = rr*sin(2*pi*u2); s(6) = 1;\n    z = rr*cos(2*pi*u2);\n",
    );
    add(
        "stream_normal3",
        "s",
        "[v, s]",
        "three normal draws, and the stream advanced.",
        "    [a, s] = asils.pc.stream_normal(s);\n    [b, s] = asils.pc.stream_normal(s);\n    [c, s] = asils.pc.stream_normal(s);\n    v = [a; b; c];\n",
    );
    f
}
