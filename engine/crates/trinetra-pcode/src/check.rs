//! The checker: the files together, as design/js/pcode.js checks them, in the same order, so the
//! same problems are named with the same words at the same places: units and dimensions, types,
//! every output set on every path, state only in a proc, constant range bounds, no recursion.
//! It also leaves on each expression what the interpreter needs (its SI value, its shape, what it
//! calls, a zero that fills an array).
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

use crate::ast::*;
use crate::error::{ErrorKind, PcodeError, Pos};
use crate::jsfmt;
use crate::units::{dim_add, dim_eq, dim_mul, dim_text, unit_of, Dim, DIMLESS};
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;

pub(crate) const BUILTINS: &[&str] = &[
    "sqrt", "abs", "sin", "cos", "tan", "asin", "acos", "atan", "atan2", "exp", "log", "log10", "min", "max", "clamp", "floor", "ceil",
    "round", "sign", "fmod", "pow", "dot", "cross", "norm", "unit", "transpose", "real", "len", "hypot", "int", "div", "rem", "band",
    "bor", "bxor", "shl", "shr", "isnan", "isfinite", "sort", "argsort", "stream", "uniform", "normal", "normal3",
];

/// The language's constants: pi, and the two values that are not finite.
pub(crate) const CONSTS: &[&str] = &["pi", "inf", "nan"];

/// Names every JavaScript object answers to (Object.prototype): the JavaScript checker finds them
/// "defined" already, so a declaration of one is refused there, and here.
const JS_OBJECT_NAMES: &[&str] = &[
    "constructor", "__defineGetter__", "__defineSetter__", "hasOwnProperty", "__lookupGetter__", "__lookupSetter__", "isPrototypeOf",
    "propertyIsEnumerable", "toString", "valueOf", "__proto__", "toLocaleString",
];

/// What a name in an expression is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum VarKind {
    #[default]
    None,
    Input,
    /// an input handed by reference: it may be changed
    InOut,
    Output,
    Local,
    State,
    Loop,
    Const(usize),
    /// a data table, by its place in `data`
    Data(usize),
    Pi,
    /// `inf`
    Inf,
    /// `nan`
    Nan,
}

/// What a `+ - * /` works on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum Shape {
    #[default]
    Scalar,
    Vec,
    Mat,
    Mv,
    Mm,
    Vs,
    Ms,
}

/// What a call calls.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub(crate) enum Target {
    #[default]
    None,
    Builtin(&'static str),
    Fn(usize),
    Table(usize),
    Record(usize),
}

/// What the checker leaves on an expression.
#[derive(Clone, Debug, Default)]
pub(crate) struct Ann {
    pub ty: Option<Ty>,
    /// a number's value in SI
    pub si: f64,
    /// a bare 0 given where an array or a record is wanted: the zero of that type
    pub fill: Option<Ty>,
    pub var: VarKind,
    pub shape: Shape,
    /// the whole-number power of `^`
    pub k: f64,
    /// `array * number` (not `number * array`)
    pub arr_left: bool,
    pub target: Target,
    /// an array literal's unit, as a scale to SI
    pub scale: Option<f64>,
    /// a range bound's value in SI
    pub bound: Option<f64>,
    /// a field's place in its record
    pub field: usize,
    /// an option of a choice (`Kind.fmr`): the choice's place in `choices` and the option's number
    pub choice: Option<(usize, usize)>,
    /// on the expression of a `let` of one name, or of a `state`: the type the name is given
    /// (the JavaScript's `s.vty`)
    pub vty: Option<Ty>,
    /// a call that is the whole right side of a let or an assignment
    pub top: bool,
    /// a call that changes its inout inputs (their values go back into the caller's variables)
    pub inout: bool,
    /// the places of a call's inout inputs
    pub inout_args: Vec<usize>,
    /// a name in quotes where one is allowed (a stream's id)
    pub ok_str: bool,
    /// `stream(seed, "name")`: the name's id, its 32-bit halves
    pub sid: Option<(f64, f64)>,
}

/// The checked program: the declarations, the expressions and what the checker left on them.
pub(crate) struct Checked {
    pub exprs: Vec<Expr>,
    pub ann: Vec<Ann>,
    pub fns: Vec<Func>,
    pub tables: Vec<Table>,
    pub records: Vec<Record>,
    pub consts: Vec<Const>,
    pub data: Vec<Data>,
    pub choices: Vec<Choice>,
    pub modules: Vec<(String, Vec<String>)>,
    /// each module's declarations (parallel to `modules`), in the order they were declared
    pub mod_items: Vec<Vec<ItemRef>>,
}

/// A declaration: its place in `fns`, `tables`, `records` or `consts`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ItemRef {
    Fn(usize),
    Table(usize),
    Record(usize),
    Const(usize),
    Data(usize),
    Choice(usize),
}

struct Checker {
    exprs: Vec<Expr>,
    ann: Vec<Ann>,
    errors: Vec<PcodeError>,
    crashed: Option<PcodeError>,
    fns: Vec<Func>,
    tables: Vec<Table>,
    records: Vec<Record>,
    consts: Vec<Const>,
    data: Vec<Data>,
    choices: Vec<Choice>,
    const_ty: Vec<Option<Ty>>,
    names: HashMap<String, ItemRef>,
    modules: Vec<(String, Vec<String>)>,
    mod_items: Vec<Vec<ItemRef>>,
    order: Vec<ItemRef>,
    scopes: Vec<HashMap<String, (Ty, VarKind)>>,
    cur: Option<usize>,
    cur_states: Vec<(String, ExprId)>,
    touched: HashSet<String>,
}

fn real0() -> Ty {
    Ty::Real(DIMLESS)
}
fn num_type(t: Option<&Ty>) -> bool {
    matches!(t, Some(Ty::Real(_)) | Some(Ty::Int))
}
/// a number's dimension (an int is plain); None for what has none
fn scalar_dim(t: &Ty) -> Option<Dim> {
    match t {
        Ty::Int => Some(DIMLESS),
        Ty::Real(d) => Some(*d),
        _ => None,
    }
}
fn elem(t: &Ty) -> &Ty {
    match t {
        Ty::Arr(_, of) => elem(of),
        _ => t,
    }
}
fn type_eq(a: &Ty, b: &Ty) -> bool {
    match (a, b) {
        (Ty::Real(x), Ty::Real(y)) => dim_eq(x, y),
        (Ty::Arr(n, x), Ty::Arr(m, y)) => n == m && type_eq(x, y),
        (Ty::Rec(x), Ty::Rec(y)) | (Ty::Choice(x), Ty::Choice(y)) => x == y,
        (Ty::Int, Ty::Int) | (Ty::Bool, Ty::Bool) | (Ty::Tuple(_), Ty::Tuple(_)) | (Ty::Stream, Ty::Stream) | (Ty::Str, Ty::Str) => true,
        _ => false,
    }
}
fn shape_eq(a: &Ty, b: &Ty) -> bool {
    match (a, b) {
        (Ty::Arr(n, x), Ty::Arr(m, y)) => n == m && shape_eq(x, y),
        (Ty::Arr(..), _) | (_, Ty::Arr(..)) => false,
        _ => num_type(Some(a)) && num_type(Some(b)),
    }
}
fn tt(t: &Ty) -> String {
    t.text()
}

impl Checker {
    fn e(&mut self, msg: String, pos: &Pos) {
        self.errors.push(PcodeError::new(ErrorKind::Check, msg, pos));
    }
    /// The JavaScript checker throws here (a TypeError, not a named problem): stop, and say so.
    fn crash(&mut self, what: &str, pos: &Pos) {
        if self.crashed.is_none() {
            self.crashed = Some(PcodeError::new(
                ErrorKind::Internal,
                format!("the checker cannot go on: {what} (design/js/pcode.js throws here)"),
                pos,
            ));
        }
    }
    fn pos(&self, e: ExprId) -> Pos {
        self.exprs[e].pos.clone()
    }
    fn lookup_var(&self, name: &str) -> Option<(Ty, VarKind)> {
        self.scopes.iter().rev().find_map(|s| s.get(name).cloned())
    }
    fn fn_ix(&self, name: &str) -> Option<usize> {
        match self.names.get(name) {
            Some(ItemRef::Fn(i)) => Some(*i),
            _ => None,
        }
    }
    fn table_ix(&self, name: &str) -> Option<usize> {
        match self.names.get(name) {
            Some(ItemRef::Table(i)) => Some(*i),
            _ => None,
        }
    }
    fn record_ix(&self, name: &str) -> Option<usize> {
        match self.names.get(name) {
            Some(ItemRef::Record(i)) => Some(*i),
            _ => None,
        }
    }
    fn const_ix(&self, name: &str) -> Option<usize> {
        match self.names.get(name) {
            Some(ItemRef::Const(i)) => Some(*i),
            _ => None,
        }
    }
    fn choice_ix(&self, name: &str) -> Option<usize> {
        match self.names.get(name) {
            Some(ItemRef::Choice(i)) => Some(*i),
            _ => None,
        }
    }
    fn data_ix(&self, name: &str) -> Option<usize> {
        match self.names.get(name) {
            Some(ItemRef::Data(i)) => Some(*i),
            _ => None,
        }
    }

    fn resolve(&mut self, t: &TypeDecl, pos: &Pos) -> Ty {
        match t {
            TypeDecl::Int => Ty::Int,
            TypeDecl::Bool => Ty::Bool,
            TypeDecl::Stream => Ty::Stream,
            TypeDecl::Real { unit } => match unit_of(unit, pos) {
                Ok((d, _)) => Ty::Real(d),
                Err(er) => {
                    self.errors.push(er);
                    real0()
                }
            },
            TypeDecl::Arr { n, of, cap } => {
                let n = match cap {
                    Some((name, cpos)) => self.capacity(name, cpos),
                    None => *n,
                };
                let of = self.resolve(of, pos);
                if of == Ty::Stream {
                    self.e("a stream is not an array's element: a record or a state holds one".into(), pos);
                }
                Ty::Arr(n, Box::new(of))
            }
            TypeDecl::Rec { name, pos: tpos } => {
                if self.choice_ix(name).is_some() {
                    return Ty::Choice(name.clone());
                }
                if self.record_ix(name).is_none() {
                    self.e(format!("no record {name}"), tpos);
                }
                Ty::Rec(name.clone())
            }
        }
    }

    /// a named capacity's length: a const that is a whole-number literal from 1
    fn capacity(&mut self, name: &str, pos: &Pos) -> usize {
        if let Some(ci) = self.const_ix(name) {
            if let ExprKind::Num { v, is_int: true, .. } = &self.exprs[self.consts[ci].e].kind {
                if *v >= 1.0 {
                    return *v as usize;
                }
            }
        }
        self.e(format!("{name}: an array's length is a whole number from 1, or a const that is one (const {name} = 8)"), pos);
        1
    }

    fn is_zero(&self, e: Option<ExprId>) -> bool {
        let Some(e) = e else { return false };
        match &self.exprs[e].kind {
            ExprKind::Num { v, unit, .. } => *v == 0.0 && unit.as_deref().is_none_or(str::is_empty),
            ExprKind::Arr { items, .. } => items.iter().all(|x| self.is_zero(Some(*x))),
            ExprKind::Neg(a) => self.is_zero(Some(*a)),
            _ => false,
        }
    }

    /// inf, -inf or nan: like a bare 0, it takes the unit of what it meets
    fn is_non_finite(&self, e: ExprId) -> bool {
        match &self.exprs[e].kind {
            ExprKind::Var(_) => matches!(self.ann[e].var, VarKind::Inf | VarKind::Nan),
            ExprKind::Neg(a) => self.is_non_finite(*a),
            _ => false,
        }
    }
    fn is_any(&self, e: Option<ExprId>) -> bool {
        self.is_zero(e) || e.is_some_and(|e| self.is_non_finite(e))
    }

    /// Can a value of type `got` (from expression e) go where `want` is wanted?
    fn fits(&mut self, want: &Ty, got: &Ty, e: Option<ExprId>) -> bool {
        if type_eq(want, got) {
            return true;
        }
        if e.is_some() && self.is_any(e) && shape_eq(want, got) {
            return true; // a bare 0 (or an array of them), inf or nan takes any dimension
        }
        if let Some(id) = e {
            if let ExprKind::Num { v, unit, .. } = &self.exprs[id].kind {
                if *v == 0.0 && unit.as_deref().is_none_or(str::is_empty) && matches!(want, Ty::Arr(..) | Ty::Rec(_)) {
                    self.ann[id].fill = Some(want.clone()); // 0 fills an array
                    return true;
                }
            }
            if let ExprKind::Arr { items, unit } = &self.exprs[id].kind {
                if unit.as_deref().is_none_or(str::is_empty) {
                    if let (Ty::Arr(wn, wof), Ty::Arr(gn, _)) = (want, got) {
                        if wn == gn {
                            let items = items.clone();
                            let mut all = true;
                            for x in items {
                                let xt = self.ann[x].ty.clone();
                                let ok = match xt {
                                    Some(xt) => self.fits(wof, &xt, Some(x)),
                                    None => {
                                        self.crash("an array item has no type", &self.pos(x));
                                        false
                                    }
                                };
                                if !ok {
                                    all = false;
                                    break;
                                }
                            }
                            if all {
                                self.ann[id].ty = Some(want.clone()); // a literal takes the element type it is given to
                                return true;
                            }
                        }
                    }
                }
            }
        }
        if let (Ty::Real(wd), Ty::Int) = (want, got) {
            if dim_eq(wd, &DIMLESS) {
                return true;
            }
        }
        if let (Ty::Real(wd), Ty::Real(_), Some(id)) = (want, got, e) {
            if let ExprKind::Num { unit, .. } = &self.exprs[id].kind {
                if unit.as_deref().is_none_or(str::is_empty) && dim_eq(wd, &DIMLESS) {
                    return true;
                }
            }
        }
        false
    }

    /// for + - and comparisons: the same dimension (a bare 0 adopts the other)
    fn unify(&mut self, a: &Ty, b: &Ty, ea: Option<ExprId>, eb: Option<ExprId>, op: &str, pos: &Pos) -> Ty {
        if self.is_any(ea) && shape_eq(a, b) {
            return b.clone();
        }
        if self.is_any(eb) && shape_eq(a, b) {
            return a.clone();
        }
        if matches!(a, Ty::Arr(..)) || matches!(b, Ty::Arr(..)) {
            if let (Ty::Arr(n, x), Ty::Arr(m, y)) = (a, b) {
                if n == m {
                    return Ty::Arr(*n, Box::new(self.unify(x, y, None, None, op, pos)));
                }
            }
            self.e(format!("{op}: {} and {} differ in shape", tt(a), tt(b)), pos);
            return a.clone();
        }
        if *a == Ty::Int && *b == Ty::Int {
            return Ty::Int;
        }
        if !num_type(Some(a)) || !num_type(Some(b)) {
            self.e(format!("{op} needs numbers, not {} and {}", tt(a), tt(b)), pos);
            return real0();
        }
        let (da, db) = (scalar_dim(a).unwrap(), scalar_dim(b).unwrap());
        if !dim_eq(&da, &db) {
            self.e(format!("{op}: the units differ, [{}] and [{}]", dim_text(&da), dim_text(&db)), pos);
        }
        Ty::Real(da)
    }

    fn ty(&mut self, e: ExprId) -> Ty {
        let t = self.ty_inner(e);
        self.ann[e].ty = Some(t.clone());
        t
    }

    fn ty_inner(&mut self, e: ExprId) -> Ty {
        let pos = self.pos(e);
        match self.exprs[e].kind.clone() {
            ExprKind::Num { v, is_int, unit } => {
                if let Some(u) = unit {
                    return match unit_of(&u, &pos) {
                        Ok((d, scale)) => {
                            self.ann[e].si = v * scale;
                            Ty::Real(d)
                        }
                        Err(er) => {
                            self.ann[e].si = f64::NAN;
                            self.errors.push(er);
                            real0()
                        }
                    };
                }
                self.ann[e].si = v;
                if is_int {
                    Ty::Int
                } else {
                    real0()
                }
            }
            ExprKind::Bool(_) => Ty::Bool,
            ExprKind::Str(_) => {
                if !self.ann[e].ok_str {
                    self.e("a name in quotes is a stream's only: stream(seed, \"gyro\")".into(), &pos);
                }
                Ty::Str
            }
            ExprKind::Var(name) => {
                if let Some((t, kind)) = self.lookup_var(&name) {
                    self.ann[e].var = kind;
                    return t;
                }
                if let Some(ci) = self.const_ix(&name) {
                    self.ann[e].var = VarKind::Const(ci);
                    return self.const_ty[ci].clone().unwrap_or_else(real0);
                }
                if let Some(di) = self.data_ix(&name) {
                    self.ann[e].var = VarKind::Data(di);
                    return self.data[di].ty.clone();
                }
                if CONSTS.contains(&name.as_str()) {
                    self.ann[e].var = match name.as_str() {
                        "pi" => VarKind::Pi,
                        "inf" => VarKind::Inf,
                        _ => VarKind::Nan,
                    };
                    return real0();
                }
                self.e(format!("{name} is not defined here (a let declares a name)"), &pos);
                real0()
            }
            ExprKind::Arr { items, unit } => self.arr_ty(e, &items, unit.as_deref(), &pos),
            ExprKind::Index { a, i } => {
                let at = self.ty(a);
                let it = self.ty(i);
                if it != Ty::Int {
                    self.e(format!("an index is an int, not {}", tt(&it)), &pos);
                }
                let Ty::Arr(n, of) = at else {
                    self.e(format!("{} cannot be indexed", tt(&at)), &pos);
                    return real0();
                };
                if let ExprKind::Num { v, .. } = &self.exprs[i].kind {
                    if *v < 0.0 || *v >= n as f64 {
                        let v = *v;
                        self.e(format!("index {} is outside 0..{}", jsfmt::num(v), n as i64 - 1), &pos);
                    }
                }
                *of
            }
            ExprKind::Field { a, f } => {
                // Kind.fmr: an option of a choice (a name no variable here has)
                if let ExprKind::Var(cn) = &self.exprs[a].kind {
                    if let (None, Some(ci)) = (self.lookup_var(cn), self.choice_ix(cn)) {
                        let c = &self.choices[ci];
                        let i = c.options.iter().position(|o| o.0 == f);
                        if i.is_none() {
                            let names: Vec<&str> = c.options.iter().map(|o| o.0.as_str()).collect();
                            let msg = format!("{} has no option {f} (its options: {})", c.name, names.join(", "));
                            self.e(msg, &pos);
                        }
                        self.ann[e].choice = Some((ci, i.unwrap_or(0)));
                        return Ty::Choice(self.choices[ci].name.clone());
                    }
                }
                let at = self.ty(a);
                let Ty::Rec(rname) = &at else {
                    self.e(format!("{} has no fields", tt(&at)), &pos);
                    return real0();
                };
                let found = self.record_ix(rname).and_then(|ri| self.records[ri].fields.iter().position(|x| x.name == f).map(|fi| (ri, fi)));
                let Some((ri, fi)) = found else {
                    self.e(format!("{rname} has no field {f}"), &pos);
                    return real0();
                };
                self.ann[e].field = fi;
                self.records[ri].fields[fi].ty.clone()
            }
            ExprKind::Not(a) => {
                let at = self.ty(a);
                if at != Ty::Bool {
                    self.e(format!("not needs a bool, not {}", tt(&at)), &pos);
                }
                Ty::Bool
            }
            ExprKind::Neg(a) => {
                let at = self.ty(a);
                if !(num_type(Some(&at)) || (matches!(at, Ty::Arr(..)) && num_type(Some(elem(&at))))) {
                    self.e(format!("- needs a number or an array of them, not {}", tt(&at)), &pos);
                }
                at
            }
            ExprKind::Ifx { c, a, b } => {
                let ct = self.ty(c);
                let at = self.ty(a);
                let bt = self.ty(b);
                if ct != Ty::Bool {
                    self.e(format!("if needs a bool, not {}", tt(&ct)), &pos);
                }
                if at == Ty::Bool || bt == Ty::Bool {
                    if !(at == Ty::Bool && bt == Ty::Bool) {
                        self.e("if: both branches are bools or neither is".into(), &pos);
                    }
                    return Ty::Bool;
                }
                if matches!(at, Ty::Rec(_) | Ty::Choice(_) | Ty::Stream) || matches!(bt, Ty::Rec(_) | Ty::Choice(_) | Ty::Stream) {
                    if !type_eq(&at, &bt) {
                        self.e("if: the branches differ".into(), &pos);
                    }
                    return at;
                }
                let t = self.unify(&at, &bt, Some(a), Some(b), "if", &pos);
                if t == Ty::Int {
                    Ty::Int
                } else {
                    t
                }
            }
            ExprKind::Bin { op, a, b } => self.bin_ty(e, op, a, b, &pos),
            ExprKind::Call { f, args } => self.call_ty(e, &f, &args, &pos),
        }
    }

    fn arr_ty(&mut self, e: ExprId, items: &[ExprId], unit: Option<&str>, pos: &Pos) -> Ty {
        if items.is_empty() {
            self.e("an empty array".into(), pos);
            return real0();
        }
        let ts: Vec<Ty> = items.iter().map(|&x| self.ty(x)).collect();
        let mut base = items.iter().zip(&ts).find(|(x, _)| !self.is_any(Some(**x))).map(|(_, t)| t.clone()).unwrap_or_else(|| ts[0].clone());
        if let Some(u) = unit.filter(|u| !u.is_empty()) {
            let (d, scale) = match unit_of(u, pos) {
                Ok(x) => x,
                Err(er) => {
                    self.errors.push(er);
                    (DIMLESS, 1.0)
                }
            };
            self.ann[e].scale = Some(scale);
            if ts.iter().any(|t| !num_type(Some(t))) {
                self.e("a unit after an array applies to numbers".into(), pos);
            }
            let plain = |t: &Ty| *t == Ty::Int || matches!(t, Ty::Real(d) if dim_eq(d, &DIMLESS));
            if items.iter().zip(&ts).any(|(x, t)| !self.is_zero(Some(*x)) && !plain(t)) {
                self.e("an array with a unit holds plain numbers: [1, 2, 3] [m]".into(), pos);
            }
            return Ty::Arr(items.len(), Box::new(Ty::Real(d)));
        }
        // items agree in shape and dimension; an int (or int array) sits beside plain reals
        fn compat(a: &Ty, b: &Ty) -> bool {
            match (a, b) {
                (Ty::Arr(n, x), Ty::Arr(m, y)) => n == m && compat(x, y),
                (Ty::Arr(..), _) | (_, Ty::Arr(..)) => false,
                _ => {
                    type_eq(a, b)
                        || (num_type(Some(a)) && num_type(Some(b)) && dim_eq(&scalar_dim(a).unwrap(), &scalar_dim(b).unwrap()))
                }
            }
        }
        fn real_of(t: &Ty) -> Ty {
            match t {
                Ty::Arr(n, of) => Ty::Arr(*n, Box::new(real_of(of))),
                Ty::Int => Ty::Real(DIMLESS),
                t => t.clone(),
            }
        }
        fn all_int(t: &Ty) -> bool {
            match t {
                Ty::Arr(_, of) => all_int(of),
                t => *t == Ty::Int,
            }
        }
        for (x, t) in items.iter().zip(&ts) {
            if !self.is_any(Some(*x)) && !compat(t, &base) {
                let p = self.pos(*x);
                self.e(format!("array items differ: {} and {}", tt(&base), tt(t)), &p);
            }
        }
        if !ts.iter().all(all_int) {
            // not every item whole: the array is real
            base = real_of(&base);
            for &x in items {
                if matches!(self.exprs[x].kind, ExprKind::Arr { .. }) {
                    if let Some(xt) = self.ann[x].ty.clone() {
                        if all_int(&xt) {
                            self.ann[x].ty = Some(real_of(&xt));
                        }
                    }
                }
            }
        }
        Ty::Arr(items.len(), Box::new(base))
    }

    fn const_int(&self, e: ExprId) -> Option<f64> {
        match &self.exprs[e].kind {
            ExprKind::Num { v, is_int: true, .. } => Some(*v),
            ExprKind::Neg(a) => match &self.exprs[*a].kind {
                ExprKind::Num { v, is_int: true, .. } => Some(-*v),
                _ => None,
            },
            _ => None,
        }
    }

    fn bin_ty(&mut self, e: ExprId, op: BinOp, ea: ExprId, eb: ExprId, pos: &Pos) -> Ty {
        let ops = op.text();
        if op == BinOp::And || op == BinOp::Or {
            let a = self.ty(ea);
            let b = self.ty(eb);
            if a != Ty::Bool || b != Ty::Bool {
                self.e(format!("{ops} needs bools, not {} and {}", tt(&a), tt(&b)), pos);
            }
            return Ty::Bool;
        }
        if op == BinOp::Pow {
            let a = self.ty(ea);
            self.ty(eb);
            if !num_type(Some(&a)) {
                self.e(format!("^ needs a number, not {}", tt(&a)), pos);
                return a;
            }
            let Some(k) = self.const_int(eb) else {
                self.e("the power after ^ is a whole-number literal (x^2, x^-3); for any other power write pow(x, y)".into(), pos);
                return a;
            };
            self.ann[e].k = k;
            if k == 0.0 {
                return if a == Ty::Int { Ty::Int } else { real0() };
            }
            if a == Ty::Int {
                return if k > 0.0 { Ty::Int } else { real0() };
            }
            let Ty::Real(d) = a else { unreachable!() };
            return Ty::Real(dim_mul(&d, k));
        }
        let a = self.ty(ea);
        let b = self.ty(eb);
        if op.is_cmp() {
            if a == Ty::Bool && b == Ty::Bool && (op == BinOp::Eq || op == BinOp::Ne) {
                return Ty::Bool;
            }
            if matches!(a, Ty::Choice(_)) || matches!(b, Ty::Choice(_)) {
                if !((op == BinOp::Eq || op == BinOp::Ne) && type_eq(&a, &b)) {
                    self.e(format!("{ops}: a choice is compared with == or != to an option of its own, not {} and {}", tt(&a), tt(&b)), pos);
                }
                return Ty::Bool;
            }
            if matches!(a, Ty::Arr(..)) || matches!(b, Ty::Arr(..)) {
                self.e(format!("{ops} compares numbers, not arrays"), pos);
            } else {
                self.unify(&a, &b, Some(ea), Some(eb), ops, pos);
            }
            return Ty::Bool;
        }
        if op == BinOp::Add || op == BinOp::Sub {
            let t = self.unify(&a, &b, Some(ea), Some(eb), ops, pos);
            self.ann[e].shape = match &t {
                Ty::Arr(_, of) if matches!(**of, Ty::Arr(..)) => Shape::Mat,
                Ty::Arr(..) => Shape::Vec,
                _ => Shape::Scalar,
            };
            return t;
        }
        // * and /
        let sign = if op == BinOp::Div { -1.0 } else { 1.0 };
        if let (Ty::Arr(an, aof), Ty::Arr(bn, bof)) = (&a, &b) {
            if op == BinOp::Div {
                self.e("/ between arrays: divide by a number, or solve".into(), pos);
                return a.clone();
            }
            if let (Ty::Arr(aofn, _), Ty::Arr(bofn, _)) = (&**aof, &**bof) {
                // matrix x matrix
                if aofn != bn {
                    self.e(format!("matrix product: {an}x{aofn} times {bn}x{bofn}"), pos);
                }
                self.ann[e].shape = Shape::Mm;
                let d = match (elem(&a), elem(&b)) {
                    (Ty::Real(x), Ty::Real(y)) => dim_add(x, y, 1.0),
                    _ => {
                        self.crash("a matrix product of an element that is not real", pos);
                        DIMLESS
                    }
                };
                return Ty::Arr(*an, Box::new(Ty::Arr(*bofn, Box::new(Ty::Real(d)))));
            }
            if let Ty::Arr(aofn, _) = &**aof {
                // matrix x vector
                if aofn != bn {
                    self.e(format!("matrix times vector: {an}x{aofn} times {bn}"), pos);
                }
                self.ann[e].shape = Shape::Mv;
                let d = match (elem(&a), scalar_dim(elem(&b))) {
                    (Ty::Real(x), Some(y)) => dim_add(x, &y, 1.0),
                    _ => {
                        self.crash("a matrix times a vector of an element that is not a number", pos);
                        DIMLESS
                    }
                };
                return Ty::Arr(*an, Box::new(Ty::Real(d)));
            }
            self.e("vector * vector: write dot(a, b), cross(a, b), or index them".into(), pos);
            return a.clone();
        }
        if matches!(a, Ty::Arr(..)) || matches!(b, Ty::Arr(..)) {
            let a_arr = matches!(a, Ty::Arr(..));
            let (arr, sc) = if a_arr { (&a, &b) } else { (&b, &a) };
            if !num_type(Some(sc)) {
                self.e(format!("{ops}: {} does not scale an array", tt(sc)), pos);
                return arr.clone();
            }
            if op == BinOp::Div && matches!(b, Ty::Arr(..)) {
                self.e("a number divided by an array".into(), pos);
                return arr.clone();
            }
            let Ty::Arr(_, of) = arr else { unreachable!() };
            self.ann[e].shape = if matches!(**of, Ty::Arr(..)) { Shape::Ms } else { Shape::Vs };
            self.ann[e].arr_left = a_arr;
            let sd = scalar_dim(sc).unwrap();
            fn scale_el(t: &Ty, sd: &Dim, sign: f64) -> Option<Ty> {
                match t {
                    Ty::Arr(n, of) => Some(Ty::Arr(*n, Box::new(scale_el(of, sd, sign)?))),
                    t => Some(Ty::Real(dim_add(&scalar_dim(t)?, sd, sign))),
                }
            }
            return match scale_el(arr, &sd, sign) {
                Some(t) => t,
                None => {
                    self.crash("an array of what is not a number, scaled", pos);
                    arr.clone()
                }
            };
        }
        if !num_type(Some(&a)) || !num_type(Some(&b)) {
            self.e(format!("{ops} needs numbers, not {} and {}", tt(&a), tt(&b)), pos);
            return real0();
        }
        self.ann[e].shape = Shape::Scalar;
        if op == BinOp::Mul && a == Ty::Int && b == Ty::Int {
            return Ty::Int;
        }
        Ty::Real(dim_add(&scalar_dim(&a).unwrap(), &scalar_dim(&b).unwrap(), sign))
    }

    fn want(&mut self, e: ExprId, t: &Ty, what: &str) -> Ty {
        let got = self.ty(e);
        if !self.fits(t, &got, Some(e)) {
            let p = self.pos(e);
            self.e(format!("{what}: wants {}, gets {}", tt(t), tt(&got)), &p);
        }
        got
    }

    fn call_ty(&mut self, e: ExprId, f: &str, args: &[ExprId], pos: &Pos) -> Ty {
        let (module, name) = match f.split_once('.') {
            Some((m, n)) => (Some(m), n),
            None => (None, f),
        };
        if let Some(m) = module {
            if !self.modules.iter().any(|x| x.0 == m) {
                self.e(format!("no module {m}"), pos);
            }
        }
        if name == "stream" {
            if let Some(&a1) = args.get(1) {
                if matches!(self.exprs[a1].kind, ExprKind::Str(_)) {
                    self.ann[a1].ok_str = true;
                }
            }
        }
        if module.is_none() {
            if let Some(b) = BUILTINS.iter().find(|b| **b == name) {
                self.ann[e].target = Target::Builtin(b);
                return self.builtin_ty(e, b, args, pos);
            }
        }
        if let Some(fi) = self.fn_ix(name) {
            self.ann[e].target = Target::Fn(fi);
            let func = &self.fns[fi];
            let (kind, params, outs) = (func.kind, func.params.clone(), func.outs.clone());
            if kind == FnKind::Proc {
                self.e(format!("{name} is a proc (it keeps state): call it from a proc, as a statement-level let with its state"), pos);
            }
            if args.len() != params.len() {
                let names: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
                self.e(format!("{name} takes {} inputs ({}), given {}", params.len(), names.join(", "), args.len()), pos);
            }
            for (a, p) in args.iter().zip(&params) {
                self.want(*a, &p.ty, &format!("{name}: input {}", p.name));
            }
            if params.iter().any(|p| p.inout) {
                let ix: Vec<(usize, String)> = params.iter().enumerate().filter(|(_, p)| p.inout).map(|(i, p)| (i, p.name.clone())).collect();
                self.inout_call(e, name, &ix, args, pos);
            }
            return if outs.len() == 1 { outs[0].ty.clone() } else { Ty::Tuple(outs.iter().map(|o| o.ty.clone()).collect()) };
        }
        if let Some(ti) = self.table_ix(name) {
            self.ann[e].target = Target::Table(ti);
            let (key, outs) = (self.tables[ti].key.ty.clone(), self.tables[ti].outs.clone());
            if args.len() != 1 {
                self.e(format!("table {name} takes one key"), pos);
            } else {
                self.want(args[0], &key, &format!("table {name}: key"));
            }
            return if outs.len() == 1 { outs[0].ty.clone() } else { Ty::Tuple(outs.iter().map(|o| o.ty.clone()).collect()) };
        }
        if let Some(ri) = self.record_ix(name) {
            if !args.is_empty() {
                self.e(format!("{name}() makes a record with every field zero; it takes no inputs"), pos);
            }
            self.ann[e].target = Target::Record(ri);
            return Ty::Rec(name.to_string());
        }
        self.e(format!("no function {f}"), pos);
        for &a in args {
            self.ty(a);
        }
        real0()
    }

    /// a call that changes its inout inputs: the whole right side of a let or an assignment, each inout input a
    /// variable the caller may change, named by no other input (so a translation may hand it over by reference)
    fn inout_call(&mut self, e: ExprId, name: &str, ix: &[(usize, String)], args: &[ExprId], pos: &Pos) {
        self.ann[e].inout = true;
        self.ann[e].inout_args = ix.iter().map(|x| x.0).collect();
        if !self.ann[e].top {
            self.e(format!("{name} changes its inout input: call it as the whole right side of a let or an assignment"), pos);
        }
        for (i, pname) in ix {
            let i = *i;
            let Some(&a) = args.get(i) else { continue };
            let var = match &self.exprs[a].kind {
                ExprKind::Var(n) => self.lookup_var(n).map(|v| (n.clone(), v.1)),
                _ => None,
            };
            let apos = self.pos(a);
            let Some((vn, kind)) = var.filter(|v| matches!(v.1, VarKind::Local | VarKind::Output | VarKind::State | VarKind::InOut)) else {
                self.e(format!("{name}: input {pname} is inout: pass it a variable (a let, an output, a state or an inout input)"), &apos);
                continue;
            };
            let _ = kind;
            if args.iter().enumerate().any(|(j, &b)| j != i && self.mentions(b, &vn)) {
                self.e(format!("{name}: {vn} is passed as inout, so no other input may name it"), &apos);
            }
        }
    }
    /// an output handed to a call as an inout input is set by it (as one set element by element is)
    fn inout_touched(&mut self, e: ExprId) {
        if !self.ann[e].inout {
            return;
        }
        if let ExprKind::Call { args, .. } = &self.exprs[e].kind {
            for &i in &self.ann[e].inout_args {
                if let Some(ExprKind::Var(n)) = args.get(i).map(|&a| &self.exprs[a].kind) {
                    self.touched.insert(n.clone());
                }
            }
        }
    }
    /// does an expression name a variable?
    fn mentions(&self, x: ExprId, n: &str) -> bool {
        let mut found = false;
        visit_expr(x, &self.exprs, &mut |y| {
            if let ExprKind::Var(m) = &self.exprs[y].kind {
                if m == n {
                    found = true;
                }
            }
        });
        found
    }

    fn builtin_ty(&mut self, e: ExprId, name: &'static str, args: &[ExprId], pos: &Pos) -> Ty {
        let ts: Vec<Ty> = args.iter().map(|&a| self.ty(a)).collect();
        let t = |i: usize| ts.get(i);
        let n = |s: &mut Self, k: usize| {
            if args.len() != k {
                s.e(format!("{name} takes {k} input(s), given {}", args.len()), pos);
            }
        };
        let sc = |s: &mut Self, t: Option<&Ty>, i: usize| -> Dim {
            match t.and_then(scalar_dim) {
                Some(d) => d,
                None => {
                    s.e(format!("{name}: input {} is a number, not {}", i + 1, type_text(t)), pos);
                    DIMLESS
                }
            }
        };
        let dl = |s: &mut Self, t: Option<&Ty>, i: usize| {
            let d = sc(s, t, i);
            if !dim_eq(&d, &DIMLESS) {
                s.e(format!("{name} takes a plain number (an angle in rad is one), not [{}]", dim_text(&d)), pos);
            }
        };
        let vec = |s: &mut Self, t: Option<&Ty>, i: usize| -> (usize, Dim) {
            if let Some(Ty::Arr(n, of)) = t {
                if let Some(d) = scalar_dim(of) {
                    return (*n, d);
                }
            }
            s.e(format!("{name}: input {} is a vector, not {}", i + 1, type_text(t)), pos);
            (3, DIMLESS)
        };
        match name {
            "sqrt" => {
                n(self, 1);
                let d = sc(self, t(0), 0);
                if d.iter().any(|x| x % 2.0 != 0.0) {
                    self.e(format!("sqrt of [{}] has no unit", dim_text(&d)), pos);
                }
                Ty::Real(dim_mul(&d, 0.5))
            }
            "abs" | "floor" | "ceil" | "round" => {
                n(self, 1);
                if t(0) == Some(&Ty::Int) {
                    return Ty::Int;
                }
                Ty::Real(sc(self, t(0), 0))
            }
            "sign" => {
                n(self, 1);
                sc(self, t(0), 0);
                real0()
            }
            "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "exp" | "log" | "log10" => {
                n(self, 1);
                dl(self, t(0), 0);
                real0()
            }
            "atan2" | "hypot" | "fmod" => {
                n(self, 2);
                let d = sc(self, t(0), 0);
                let d2 = sc(self, t(1), 1);
                if !dim_eq(&d, &d2) {
                    if args.len() < 2 {
                        self.crash(&format!("{name} with one input of a unit"), pos);
                        return real0();
                    }
                    if !self.is_any(Some(args[1])) && !self.is_any(Some(args[0])) {
                        self.e(format!("{name}: the inputs differ in unit, [{}] and [{}]", dim_text(&d), dim_text(&d2)), pos);
                    }
                }
                if name == "atan2" {
                    real0()
                } else {
                    Ty::Real(d)
                }
            }
            "pow" => {
                n(self, 2);
                dl(self, t(0), 0);
                dl(self, t(1), 1);
                real0()
            }
            "min" | "max" => {
                if args.len() < 2 {
                    self.e(format!("{name} takes two inputs or more"), pos);
                }
                if ts.is_empty() {
                    self.crash(&format!("{name}() with no inputs"), pos);
                    return real0();
                }
                let mut r = ts[0].clone();
                for i in 1..ts.len() {
                    r = self.unify(&r, &ts[i], Some(args[0]), Some(args[i]), name, pos);
                }
                if matches!(r, Ty::Arr(..)) {
                    self.e(format!("{name} takes numbers"), pos);
                }
                if ts.iter().all(|x| *x == Ty::Int) {
                    Ty::Int
                } else {
                    r
                }
            }
            "clamp" => {
                n(self, 3);
                if args.len() < 3 {
                    self.crash("clamp with fewer than three inputs", pos);
                    return real0();
                }
                let r = self.unify(&ts[0], &ts[1], Some(args[0]), Some(args[1]), name, pos);
                let r = self.unify(&r, &ts[2], Some(args[0]), Some(args[2]), name, pos);
                if r == Ty::Int {
                    Ty::Int
                } else {
                    r
                }
            }
            "real" => {
                n(self, 1);
                if t(0) != Some(&Ty::Int) {
                    self.e("real() turns an int into a real".into(), pos);
                }
                real0()
            }
            "isnan" | "isfinite" => {
                n(self, 1);
                sc(self, t(0), 0);
                Ty::Bool
            }
            "int" => {
                n(self, 1);
                if matches!(t(0), Some(Ty::Choice(_))) {
                    return Ty::Int;
                }
                dl(self, t(0), 0);
                if matches!(t(0), Some(Ty::Arr(..))) {
                    self.e("int() takes a number".into(), pos);
                }
                Ty::Int
            }
            "band" | "bor" | "bxor" | "shl" | "shr" => {
                n(self, 2);
                if !(t(0) == Some(&Ty::Int) && t(1) == Some(&Ty::Int)) {
                    self.e(format!("{name} takes two ints (non-negative, below 2^53)"), pos);
                }
                Ty::Int
            }
            "div" | "rem" => {
                n(self, 2);
                if !(t(0) == Some(&Ty::Int) && t(1) == Some(&Ty::Int)) {
                    self.e(format!("{name} takes two ints (whole-number division, truncated as in C)"), pos);
                }
                Ty::Int
            }
            "len" => {
                n(self, 1);
                if !matches!(t(0), Some(Ty::Arr(..))) {
                    self.e("len() takes an array".into(), pos);
                }
                Ty::Int
            }
            "dot" => {
                n(self, 2);
                let a = vec(self, t(0), 0);
                let b = vec(self, t(1), 1);
                if a.0 != b.0 {
                    self.e(format!("dot: lengths {} and {}", a.0, b.0), pos);
                }
                Ty::Real(dim_add(&a.1, &b.1, 1.0))
            }
            "cross" => {
                n(self, 2);
                let a = vec(self, t(0), 0);
                let b = vec(self, t(1), 1);
                if a.0 != 3 || b.0 != 3 {
                    self.e("cross takes two 3-vectors".into(), pos);
                }
                Ty::Arr(3, Box::new(Ty::Real(dim_add(&a.1, &b.1, 1.0))))
            }
            // the toolbox sort: ascending and stable, the same insertion sort in every translation
            "sort" => {
                n(self, 1);
                let a = vec(self, t(0), 0);
                match t(0) {
                    Some(x @ Ty::Arr(_, of)) if !matches!(**of, Ty::Arr(..)) && num_type(Some(of)) => x.clone(),
                    _ => Ty::Arr(a.0, Box::new(Ty::Real(a.1))),
                }
            }
            "argsort" => {
                n(self, 1);
                let a = vec(self, t(0), 0);
                Ty::Arr(a.0, Box::new(Ty::Int))
            }
            // random streams (the toolbox's: adcs-sim-core rng.rs): a stream by its seed and its number or name; a
            // draw advances the stream it is given, an inout input
            "stream" => {
                n(self, 2);
                if !(t(0) == Some(&Ty::Int) && matches!(t(1), Some(Ty::Int) | Some(Ty::Str))) {
                    self.e("stream(seed, id): the seed is an int, the id an int or a name in quotes".into(), pos);
                }
                if let Some(&a1) = args.get(1) {
                    if let ExprKind::Str(s) = &self.exprs[a1].kind {
                        let (hi, lo) = crate::interp::fnv1a(s);
                        self.ann[e].sid = Some((hi, lo));
                    }
                }
                Ty::Stream
            }
            "uniform" | "normal" | "normal3" => {
                n(self, 1);
                if t(0) != Some(&Ty::Stream) {
                    self.e(format!("{name} draws from a stream, not {}", type_text(t(0))), pos);
                }
                self.inout_call(e, name, &[(0, "stream".to_string())], args, pos);
                if name == "normal3" {
                    Ty::Arr(3, Box::new(real0()))
                } else {
                    real0()
                }
            }
            "norm" => {
                n(self, 1);
                let a = vec(self, t(0), 0);
                Ty::Real(a.1)
            }
            "unit" => {
                n(self, 1);
                let a = vec(self, t(0), 0);
                Ty::Arr(a.0, Box::new(real0()))
            }
            "transpose" => {
                n(self, 1);
                match t(0) {
                    Some(Ty::Arr(rows, of)) => match &**of {
                        Ty::Arr(cols, el) => Ty::Arr(*cols, Box::new(Ty::Arr(*rows, el.clone()))),
                        _ => {
                            self.e("transpose takes a matrix".into(), pos);
                            ts[0].clone()
                        }
                    },
                    Some(other) => {
                        let other = other.clone();
                        self.e("transpose takes a matrix".into(), pos);
                        other
                    }
                    None => {
                        self.e("transpose takes a matrix".into(), pos);
                        self.crash("transpose() of nothing has no type", pos);
                        real0()
                    }
                }
            }
            _ => {
                self.e(format!("no builtin {name}"), pos);
                real0()
            }
        }
    }

    // statements; `assigned` holds the outputs certainly set on every path
    fn check_block(&mut self, stmts: &[Stmt], assigned: &mut HashSet<String>, in_loop: bool) {
        self.scopes.push(HashMap::new());
        for s in stmts {
            self.check_stmt(s, assigned, in_loop);
        }
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str, t: Ty, pos: &Pos, kind: VarKind) {
        if self.lookup_var(name).is_some() {
            self.e(format!("{name} is already defined here; a name is declared once"), pos);
        }
        self.scopes.last_mut().unwrap().insert(name.to_string(), (t, kind));
    }

    fn check_stmt(&mut self, s: &Stmt, assigned: &mut HashSet<String>, in_loop: bool) {
        let pos = &s.pos;
        match &s.kind {
            StmtKind::Let { names, decl, e } => {
                if matches!(self.exprs[*e].kind, ExprKind::Call { .. }) {
                    self.ann[*e].top = true;
                }
                let t = self.ty(*e);
                self.inout_touched(*e);
                let decl_ty = decl.as_ref().map(|d| self.resolve(d, pos));
                if names.len() > 1 {
                    match &t {
                        Ty::Tuple(items) if items.len() == names.len() => {
                            for (n, it) in names.iter().zip(items) {
                                self.declare(n, it.clone(), pos, VarKind::Local);
                            }
                        }
                        _ => {
                            let gives = if let Ty::Tuple(items) = &t { items.len() } else { 1 };
                            self.e(format!("let {}: the right side gives {gives} value(s)", names.join(", ")), pos);
                            for n in names {
                                self.declare(n, real0(), pos, VarKind::Local);
                            }
                        }
                    }
                    return;
                }
                if let Ty::Tuple(items) = &t {
                    let f = match &self.exprs[*e].kind {
                        ExprKind::Call { f, .. } => f.clone(),
                        _ => "undefined".into(),
                    };
                    self.e(format!("{f} gives {} values: let one name for each", items.len()), pos);
                    let first = items.first().cloned();
                    match first {
                        Some(t0) => self.declare(&names[0], t0, pos, VarKind::Local),
                        None => self.crash("a call that gives no values", pos),
                    }
                    return;
                }
                if let Some(dt) = &decl_ty {
                    if !self.fits(dt, &t, Some(*e)) {
                        self.e(format!("let {}: declared {}, given {}", names[0], tt(dt), tt(&t)), pos);
                    }
                }
                let vt = decl_ty.clone().unwrap_or(t);
                if decl_ty.is_none() && self.is_zero(Some(*e)) && vt != Ty::Int {
                    self.e(format!("let {0} = 0 needs its type: let {0}: real[unit] = 0", names[0]), pos);
                }
                self.ann[*e].vty = Some(vt.clone());
                self.declare(&names[0], vt, pos, VarKind::Local);
            }
            StmtKind::State { name, decl, e } => {
                let is_proc = self.cur.is_some_and(|c| self.fns[c].kind == FnKind::Proc);
                if !is_proc {
                    self.e("state is kept by a proc, not a fn".into(), pos);
                }
                if self.scopes.len() != 2 {
                    self.e("state is declared at the top of the proc's body".into(), pos);
                }
                let st = self.resolve(decl, pos);
                let t = self.ty(*e);
                self.ann[*e].vty = Some(st.clone());
                if !self.fits(&st, &t, Some(*e)) {
                    self.e(format!("state {name}: declared {}, starts at {}", tt(&st), tt(&t)), pos);
                }
                if !self.is_const_expr(*e) {
                    self.e(format!("state {name} starts at a constant"), pos);
                }
                self.cur_states.push((name.clone(), *e));
                self.declare(name, st, pos, VarKind::State);
            }
            StmtKind::Set { targets, e } => {
                if matches!(self.exprs[*e].kind, ExprKind::Call { .. }) {
                    self.ann[*e].top = true;
                }
                let t = self.ty(*e);
                let tts: Vec<Option<Ty>> = targets.iter().map(|lv| self.lv_ty(*lv)).collect();
                self.inout_touched(*e);
                if self.ann[*e].inout {
                    let inout_vars: Vec<String> = match &self.exprs[*e].kind {
                        ExprKind::Call { args, .. } => self.ann[*e]
                            .inout_args
                            .iter()
                            .filter_map(|&i| args.get(i))
                            .filter_map(|&a| if let ExprKind::Var(n) = &self.exprs[a].kind { Some(n.clone()) } else { None })
                            .collect(),
                        _ => Vec::new(),
                    };
                    for &lv in targets {
                        if inout_vars.contains(&self.root_var(lv)) {
                            let name = self.lv_name(lv);
                            self.e(format!("{name}: the target is also the call's inout input"), pos);
                        }
                    }
                }
                if targets.len() > 1 {
                    match &t {
                        Ty::Tuple(items) if items.len() == targets.len() => {
                            for (i, (tt0, it)) in tts.iter().zip(items).enumerate() {
                                if let Some(tt0) = tt0 {
                                    if !type_eq(tt0, it) {
                                        self.e(format!("target {}: {}, given {}", i + 1, tt(tt0), tt(it)), pos);
                                    }
                                }
                            }
                        }
                        _ => {
                            let gives = if let Ty::Tuple(items) = &t { items.len() } else { 1 };
                            self.e(format!("{} targets, the right side gives {gives}", targets.len()), pos);
                        }
                    }
                } else if let Some(t0) = &tts[0] {
                    if !self.fits(t0, &t, Some(*e)) {
                        let name = self.lv_name(targets[0]);
                        self.e(format!("{name}: is {}, given {}", tt(t0), tt(&t)), pos);
                    }
                }
                for &lv in targets {
                    if let ExprKind::Var(n) = &self.exprs[lv].kind {
                        assigned.insert(n.clone());
                    } else {
                        let r = self.root_var(lv);
                        self.touched.insert(r);
                    }
                }
            }
            StmtKind::If { arms, els } => {
                let mut sets: Vec<HashSet<String>> = Vec::new();
                for (c, body) in arms {
                    let ct = self.ty(*c);
                    if ct != Ty::Bool {
                        let p = self.pos(*c);
                        self.e(format!("if needs a bool, not {}", tt(&ct)), &p);
                    }
                    let mut a = assigned.clone();
                    self.check_block(body, &mut a, in_loop);
                    sets.push(a);
                }
                if let Some(els) = els {
                    let mut a = assigned.clone();
                    self.check_block(els, &mut a, in_loop);
                    sets.push(a);
                    let first: Vec<String> = sets[0].iter().cloned().collect();
                    for n in first {
                        if sets.iter().all(|x| x.contains(&n)) {
                            assigned.insert(n);
                        }
                    }
                }
            }
            StmtKind::For { v, a, b, body } => {
                let at = self.ty(*a);
                let bt = self.ty(*b);
                if at != Ty::Int || bt != Ty::Int {
                    self.e("for runs over ints: for i in 0 .. n".into(), pos);
                }
                self.scopes.push(HashMap::new());
                self.declare(v, Ty::Int, pos, VarKind::Loop);
                let mut inner = assigned.clone();
                self.check_block(body, &mut inner, true);
                self.scopes.pop();
            }
            StmtKind::Settle { n, c, body, els } => {
                let nt = self.ty(*n);
                if nt != Ty::Int {
                    self.e("settle max N: N is an int".into(), pos);
                }
                let mut inner = assigned.clone();
                self.check_block(body, &mut inner, true);
                assigned.extend(inner); // the body runs at least once
                self.scopes.push(HashMap::new());
                let ct = self.ty(*c);
                if ct != Ty::Bool {
                    self.e(format!("until needs a bool, not {}", tt(&ct)), pos);
                }
                self.scopes.pop();
                if let Some(els) = els {
                    let mut a = assigned.clone();
                    self.check_block(els, &mut a, in_loop);
                }
            }
        }
    }

    fn root_var(&self, lv: ExprId) -> String {
        match &self.exprs[lv].kind {
            ExprKind::Var(n) => n.clone(),
            ExprKind::Index { a, .. } | ExprKind::Field { a, .. } => self.root_var(*a),
            _ => String::new(),
        }
    }
    fn lv_name(&self, lv: ExprId) -> String {
        match &self.exprs[lv].kind {
            ExprKind::Var(n) => n.clone(),
            ExprKind::Field { a, f } => format!("{}.{f}", self.lv_name(*a)),
            ExprKind::Index { a, .. } => format!("{}[…]", self.lv_name(*a)),
            _ => String::new(),
        }
    }
    fn lv_ty(&mut self, lv: ExprId) -> Option<Ty> {
        let pos = self.pos(lv);
        let t = match self.exprs[lv].kind.clone() {
            ExprKind::Var(name) => {
                let Some((t, kind)) = self.lookup_var(&name) else {
                    self.e(format!("{name} is not declared: let {name} = …"), &pos);
                    return None;
                };
                if kind == VarKind::Input {
                    self.e(format!("{name} is an input; inputs are not changed (copy it: let x = {name})"), &pos);
                }
                if kind == VarKind::Loop {
                    self.e(format!("{name} is the loop's counter"), &pos);
                }
                self.ann[lv].var = kind;
                t
            }
            ExprKind::Index { a, i } => {
                let base = self.lv_ty(a)?;
                let it = self.ty(i);
                if it != Ty::Int {
                    self.e("an index is an int".into(), &pos);
                }
                let Ty::Arr(_, of) = base else {
                    self.e(format!("{} cannot be indexed", tt(&base)), &pos);
                    return None;
                };
                *of
            }
            ExprKind::Field { a, f } => {
                let base = self.lv_ty(a)?;
                let Ty::Rec(rname) = &base else {
                    self.e(format!("{} has no fields", tt(&base)), &pos);
                    return None;
                };
                let found = self.record_ix(rname).and_then(|ri| self.records[ri].fields.iter().position(|x| x.name == f).map(|fi| (ri, fi)));
                let Some((ri, fi)) = found else {
                    self.e(format!("{rname} has no field {f}"), &pos);
                    return None;
                };
                self.ann[lv].field = fi;
                self.records[ri].fields[fi].ty.clone()
            }
            _ => return None,
        };
        self.ann[lv].ty = Some(t.clone());
        Some(t)
    }

    /// the SI value of a constant bound (a literal, maybe negated, or pi, and their products)
    fn const_num(&mut self, e: ExprId) -> Option<f64> {
        match self.exprs[e].kind.clone() {
            ExprKind::Num { .. } => Some(self.ann[e].si),
            ExprKind::Neg(a) => self.const_num(a).map(|v| -v),
            ExprKind::Var(_) if self.ann[e].var == VarKind::Pi => Some(PI),
            ExprKind::Var(_) if self.ann[e].var == VarKind::Inf => Some(f64::INFINITY),
            ExprKind::Var(_) if self.ann[e].var == VarKind::Nan => Some(f64::NAN),
            ExprKind::Bin { op: op @ (BinOp::Mul | BinOp::Div), a, b } => {
                let x = self.const_num(a);
                let y = self.const_num(b);
                match (x, y) {
                    (Some(x), Some(y)) => Some(if op == BinOp::Mul { x * y } else { x / y }),
                    _ => None,
                }
            }
            _ => {
                let p = self.pos(e);
                self.e("a range bound is a constant".into(), &p);
                None
            }
        }
    }

    fn is_const_expr(&self, e: ExprId) -> bool {
        match &self.exprs[e].kind {
            ExprKind::Num { .. } | ExprKind::Bool(_) => true,
            ExprKind::Arr { items, .. } => items.iter().all(|x| self.is_const_expr(*x)),
            ExprKind::Not(a) | ExprKind::Neg(a) => self.is_const_expr(*a),
            ExprKind::Bin { a, b, .. } => self.is_const_expr(*a) && self.is_const_expr(*b),
            ExprKind::Var(_) => matches!(self.ann[e].var, VarKind::Const(_) | VarKind::Pi | VarKind::Inf | VarKind::Nan | VarKind::Data(_)),
            ExprKind::Field { .. } => self.ann[e].choice.is_some(),
            ExprKind::Str(_) => true,
            ExprKind::Call { args, .. } => match self.ann[e].target {
                Target::Record(_) => true,
                Target::Builtin(_) => args.iter().all(|x| self.is_const_expr(*x)),
                _ => false,
            },
            _ => false,
        }
    }

    /// An input's (or a field's) range: constant bounds, in the input's own unit when written bare.
    fn check_range(&mut self, owner: &str, p: &Param) {
        let Some((lo, hi)) = p.range else { return };
        self.scopes.clear();
        self.scopes.push(HashMap::new());
        let base_ty = elem(&p.ty).clone();
        let scale = match p.decl.elem_unit() {
            Some(u) => unit_of(u, &p.pos).map(|x| x.1).unwrap_or(1.0),
            None => 1.0,
        };
        for r in [lo, hi] {
            let t = self.ty(r);
            if self.plain(r) && num_type(Some(&base_ty)) {
                let v = self.const_num(r);
                self.ann[r].bound = v.map(|v| v * scale);
                continue;
            }
            if !self.fits(&base_ty, &t, Some(r)) {
                let pp = self.pos(r);
                self.e(format!("{owner}: the range of {} is {}, not {}", p.name, tt(&base_ty), tt(&t)), &pp);
            }
            self.ann[r].bound = self.const_num(r);
        }
        if let Some(b0) = self.ann[lo].bound {
            // a missing upper bound compares as JavaScript's null does: as 0
            if !(b0 < self.ann[hi].bound.unwrap_or(0.0)) {
                self.e(format!("{owner}: the range of {} runs low .. high", p.name), &p.pos);
            }
        }
    }
    fn plain(&self, e: ExprId) -> bool {
        match &self.exprs[e].kind {
            ExprKind::Num { unit, .. } => unit.as_deref().is_none_or(str::is_empty),
            ExprKind::Neg(a) => self.plain(*a),
            _ => false,
        }
    }

    /// the functions a body calls, in the order the JavaScript walks its tree
    fn walk_calls_stmts(&self, stmts: &[Stmt], out: &mut Vec<String>) {
        for s in stmts {
            match &s.kind {
                StmtKind::Let { e, .. } | StmtKind::State { e, .. } => self.walk_calls(*e, out),
                StmtKind::Set { targets, e } => {
                    for t in targets {
                        self.walk_calls(*t, out);
                    }
                    self.walk_calls(*e, out);
                }
                StmtKind::If { arms, els } => {
                    for (c, body) in arms {
                        self.walk_calls(*c, out);
                        self.walk_calls_stmts(body, out);
                    }
                    if let Some(els) = els {
                        self.walk_calls_stmts(els, out);
                    }
                }
                StmtKind::For { a, b, body, .. } => {
                    self.walk_calls(*a, out);
                    self.walk_calls(*b, out);
                    self.walk_calls_stmts(body, out);
                }
                StmtKind::Settle { n, c, body, els } => {
                    self.walk_calls(*n, out);
                    self.walk_calls(*c, out);
                    self.walk_calls_stmts(body, out);
                    if let Some(els) = els {
                        self.walk_calls_stmts(els, out);
                    }
                }
            }
        }
    }
    fn walk_calls(&self, e: ExprId, out: &mut Vec<String>) {
        match &self.exprs[e].kind {
            ExprKind::Num { .. } | ExprKind::Bool(_) | ExprKind::Var(_) | ExprKind::Str(_) => {}
            ExprKind::Arr { items, .. } => items.iter().for_each(|x| self.walk_calls(*x, out)),
            ExprKind::Index { a, i } => {
                self.walk_calls(*a, out);
                self.walk_calls(*i, out);
            }
            ExprKind::Field { a, .. } | ExprKind::Not(a) | ExprKind::Neg(a) => self.walk_calls(*a, out),
            ExprKind::Ifx { c, a, b } => {
                self.walk_calls(*c, out);
                self.walk_calls(*a, out);
                self.walk_calls(*b, out);
            }
            ExprKind::Bin { a, b, .. } => {
                self.walk_calls(*a, out);
                self.walk_calls(*b, out);
            }
            ExprKind::Call { args, .. } => {
                if let Target::Fn(fi) = self.ann[e].target {
                    let n = &self.fns[fi].name;
                    if !out.contains(n) {
                        out.push(n.clone());
                    }
                }
                args.iter().for_each(|x| self.walk_calls(*x, out));
            }
        }
    }

    fn run(mut self, files: Vec<File>) -> Result<Checked, Vec<PcodeError>> {
        // 1 declarations
        for f in files {
            let module = f.module.clone().unwrap_or_else(|| "main".into());
            let mi = match self.modules.iter().position(|m| m.0 == module) {
                Some(mi) => mi,
                None => {
                    self.modules.push((module.clone(), f.doc.clone()));
                    self.mod_items.push(Vec::new());
                    self.modules.len() - 1
                }
            };
            for mut it in f.items {
                it.set_module(&module);
                let name = it.name().to_string();
                if self.names.contains_key(&name) || BUILTINS.contains(&name.as_str()) || CONSTS.contains(&name.as_str()) || JS_OBJECT_NAMES.contains(&name.as_str()) {
                    let p = it.pos().clone();
                    self.e(format!("{name} is defined twice (or is a builtin)"), &p);
                    continue;
                }
                let r = match it {
                    Item::Func(x) => {
                        self.fns.push(x);
                        ItemRef::Fn(self.fns.len() - 1)
                    }
                    Item::Table(x) => {
                        self.tables.push(x);
                        ItemRef::Table(self.tables.len() - 1)
                    }
                    Item::Record(x) => {
                        self.records.push(x);
                        ItemRef::Record(self.records.len() - 1)
                    }
                    Item::Const(x) => {
                        self.consts.push(x);
                        self.const_ty.push(None);
                        ItemRef::Const(self.consts.len() - 1)
                    }
                    Item::Data(x) => {
                        self.data.push(x);
                        ItemRef::Data(self.data.len() - 1)
                    }
                    Item::Choice(x) => {
                        self.choices.push(x);
                        ItemRef::Choice(self.choices.len() - 1)
                    }
                };
                self.names.insert(name, r);
                self.mod_items[mi].push(r);
                self.order.push(r);
            }
        }
        // 2 types
        for ci in 0..self.choices.len() {
            let c = self.choices[ci].clone();
            let mut seen: Vec<&str> = Vec::new();
            for (o, opos) in &c.options {
                if seen.contains(&o.as_str()) {
                    self.e(format!("choice {}: {o} is named twice", c.name), opos);
                }
                seen.push(o);
            }
        }
        for ri in 0..self.records.len() {
            for fi in 0..self.records[ri].fields.len() {
                let (decl, pos) = (self.records[ri].fields[fi].decl.clone(), self.records[ri].fields[fi].pos.clone());
                self.records[ri].fields[fi].ty = self.resolve(&decl, &pos);
                if self.records[ri].fields[fi].inout {
                    let (r, f) = (self.records[ri].name.clone(), self.records[ri].fields[fi].name.clone());
                    self.e(format!("record {r}: {f}: only an input of a fn or proc is inout"), &pos);
                }
            }
        }
        for oi in 0..self.order.len() {
            match self.order[oi] {
                ItemRef::Fn(fi) => {
                    for k in 0..self.fns[fi].params.len() {
                        let (d, p) = (self.fns[fi].params[k].decl.clone(), self.fns[fi].params[k].pos.clone());
                        let t = self.resolve(&d, &p);
                        self.fns[fi].params[k].ty = t.clone();
                        if self.fns[fi].params[k].inout && !matches!(t, Ty::Arr(..) | Ty::Rec(_) | Ty::Stream) {
                            let (f, n) = (self.fns[fi].name.clone(), self.fns[fi].params[k].name.clone());
                            self.e(format!("{f}: {n}: an inout input is an array, a record or a stream (a number is copied, not shared)"), &p);
                        }
                    }
                    for k in 0..self.fns[fi].outs.len() {
                        let (d, p) = (self.fns[fi].outs[k].decl.clone(), self.fns[fi].outs[k].pos.clone());
                        self.fns[fi].outs[k].ty = self.resolve(&d, &p);
                        if self.fns[fi].outs[k].inout {
                            let (f, n) = (self.fns[fi].name.clone(), self.fns[fi].outs[k].name.clone());
                            self.e(format!("{f}: {n}: only an input of a fn or proc is inout"), &p);
                        }
                    }
                    let f = &self.fns[fi];
                    let names: Vec<&str> = f.params.iter().chain(&f.outs).map(|x| x.name.as_str()).collect();
                    let dup = names.iter().enumerate().find(|(i, n)| names.iter().position(|m| m == *n) != Some(*i)).map(|(_, n)| n.to_string());
                    if let Some(dup) = dup {
                        let (name, pos) = (f.name.clone(), f.pos.clone());
                        self.e(format!("{name}: {dup} is named twice among the inputs and outputs"), &pos);
                    }
                }
                ItemRef::Table(ti) => self.check_table(ti),
                ItemRef::Data(di) => self.check_data(di),
                _ => {}
            }
            if self.crashed.is_some() {
                break;
            }
        }
        // 3 bodies
        for ci in 0..self.consts.len() {
            if self.crashed.is_some() {
                break;
            }
            self.scopes.clear();
            self.scopes.push(HashMap::new());
            let e = self.consts[ci].e;
            let t = self.ty(e);
            self.const_ty[ci] = Some(t);
            if !self.is_const_expr(e) {
                let (name, pos) = (self.consts[ci].name.clone(), self.consts[ci].pos.clone());
                self.e(format!("const {name} is a constant expression"), &pos);
            }
        }
        for oi in 0..self.order.len() {
            if self.crashed.is_some() {
                break;
            }
            let ItemRef::Fn(fi) = self.order[oi] else { continue };
            self.cur = Some(fi);
            self.cur_states.clear();
            self.touched.clear();
            self.scopes.clear();
            let mut top = HashMap::new();
            for p in &self.fns[fi].params {
                top.insert(p.name.clone(), (p.ty.clone(), if p.inout { VarKind::InOut } else { VarKind::Input }));
            }
            for o in &self.fns[fi].outs {
                top.insert(o.name.clone(), (o.ty.clone(), VarKind::Output));
            }
            self.scopes.push(top);
            let mut assigned = HashSet::new();
            let body = self.fns[fi].body.clone();
            self.check_block(&body, &mut assigned, false);
            self.fns[fi].states = std::mem::take(&mut self.cur_states);
            let f = &self.fns[fi];
            let missing: Vec<String> = f
                .outs
                .iter()
                .filter(|o| !(assigned.contains(&o.name) || (self.touched.contains(&o.name) && matches!(o.ty, Ty::Arr(..) | Ty::Rec(_) | Ty::Tuple(_) | Ty::Choice(_)))))
                .map(|o| o.name.clone())
                .collect();
            let (name, pos) = (f.name.clone(), f.pos.clone());
            for o in missing {
                self.e(format!("{name}: output {o} is not set on every path"), &pos);
            }
            let params = self.fns[fi].params.clone();
            for p in &params {
                self.check_range(&name, p);
            }
        }
        for ri in 0..self.records.len() {
            if self.crashed.is_some() {
                break;
            }
            let (name, fields) = (self.records[ri].name.clone(), self.records[ri].fields.clone());
            for f in &fields {
                self.check_range(&name, f);
            }
        }
        self.cur = None;
        // calls between fns: no recursion (flight code)
        let mut calls: Vec<Vec<usize>> = vec![Vec::new(); self.fns.len()];
        let fn_order: Vec<usize> = self.order.iter().filter_map(|r| if let ItemRef::Fn(i) = r { Some(*i) } else { None }).collect();
        for &fi in &fn_order {
            let mut names = Vec::new();
            let body = self.fns[fi].body.clone();
            self.walk_calls_stmts(&body, &mut names);
            calls[fi] = names.iter().filter_map(|n| self.fn_ix(n)).collect();
        }
        let mut seen = vec![false; self.fns.len()];
        let mut on_stack = vec![false; self.fns.len()];
        fn dfs(c: &mut Checker, calls: &[Vec<usize>], seen: &mut [bool], on_stack: &mut [bool], n: usize, path: &mut Vec<usize>) {
            if on_stack[n] {
                let mut names: Vec<&str> = path.iter().map(|&i| c.fns[i].name.as_str()).collect();
                names.push(&c.fns[n].name);
                let msg = format!("recursion: {} (flight code does not recurse)", names.join(" -> "));
                let pos = c.fns[n].pos.clone();
                c.e(msg, &pos);
                return;
            }
            if seen[n] {
                return;
            }
            seen[n] = true;
            on_stack[n] = true;
            path.push(n);
            for &m in &calls[n] {
                dfs(c, calls, seen, on_stack, m, path);
            }
            path.pop();
            on_stack[n] = false;
        }
        if self.crashed.is_none() {
            for &fi in &fn_order {
                dfs(&mut self, &calls, &mut seen, &mut on_stack, fi, &mut Vec::new());
            }
        }
        if let Some(c) = self.crashed {
            return Err(vec![c]);
        }
        if !self.errors.is_empty() {
            return Err(self.errors);
        }
        Ok(Checked {
            exprs: self.exprs,
            ann: self.ann,
            fns: self.fns,
            tables: self.tables,
            records: self.records,
            consts: self.consts,
            data: self.data,
            choices: self.choices,
            modules: self.modules,
            mod_items: self.mod_items,
        })
    }

    /// a data table: a 1-D or 2-D array of reals or ints, its values row by row (the outer index first), in SI
    fn check_data(&mut self, di: usize) {
        let (decl, pos, name) = (self.data[di].decl.clone(), self.data[di].pos.clone(), self.data[di].name.clone());
        let ty = self.resolve(&decl, &pos);
        self.data[di].ty = ty.clone();
        let mut dims = Vec::new();
        let mut el = &ty;
        while let Ty::Arr(n, of) = el {
            dims.push(*n);
            el = of;
        }
        if !(dims.len() == 1 || dims.len() == 2) || !matches!(el, Ty::Real(_) | Ty::Int) {
            self.e(format!("data {name}: a data table is a 1-D or 2-D array of reals or ints"), &pos);
            return;
        }
        let flat: Vec<f64> = self.data[di].rows.iter().flat_map(|(r, _)| r.iter().copied()).collect();
        let want = if dims.len() == 1 { dims[0] } else { dims[0] * dims[1] };
        if flat.len() != want {
            self.e(format!("data {name}: {} values, the type holds {want}", flat.len()), &pos);
            return;
        }
        let is_int = *el == Ty::Int;
        if is_int && !flat.iter().all(|x| x.is_finite() && x.trunc() == *x) {
            self.e(format!("data {name}: an int table holds whole numbers"), &pos);
            return;
        }
        let scale = if is_int { 1.0 } else { decl.elem_unit().and_then(|u| unit_of(u, &pos).ok()).map_or(1.0, |x| x.1) };
        self.data[di].si = Some(if is_int { flat } else { flat.iter().map(|x| x * scale).collect() });
    }

    fn check_table(&mut self, ti: usize) {
        if self.tables[ti].key.inout || self.tables[ti].outs.iter().any(|o| o.inout) {
            let (n, p) = (self.tables[ti].name.clone(), self.tables[ti].pos.clone());
            self.e(format!("table {n}: only an input of a fn or proc is inout"), &p);
        }
        let (kd, kp) = (self.tables[ti].key.decl.clone(), self.tables[ti].key.pos.clone());
        self.tables[ti].key.ty = self.resolve(&kd, &kp);
        let name = self.tables[ti].name.clone();
        for k in 0..self.tables[ti].outs.len() {
            let (d, p) = (self.tables[ti].outs[k].decl.clone(), self.tables[ti].outs[k].pos.clone());
            let t = self.resolve(&d, &p);
            if !matches!(t, Ty::Real(_)) {
                self.e(format!("table {name}: every column is real"), &p);
            }
            self.tables[ti].outs[k].ty = t;
        }
        let tab = self.tables[ti].clone();
        match (&tab.key.ty, &tab.outs[0].ty) {
            (Ty::Real(kd), Ty::Real(od)) => {
                if !dim_eq(kd, od) {
                    self.e(format!("table {name}: the first column is the key, so it has the key's dimension"), &tab.outs[0].pos);
                }
            }
            (Ty::Real(_), _) => {
                self.crash("a table whose first column is not real", &tab.outs[0].pos);
                return;
            }
            _ => self.e(format!("table {name}: the key is real"), &tab.key.pos),
        }
        let mut scales = Vec::new();
        for o in &tab.outs {
            let TypeDecl::Real { unit } = &o.decl else {
                self.crash("a table column that is not real has no unit", &o.pos);
                return;
            };
            match unit_of(unit, &o.pos) {
                Ok((_, s)) => scales.push(s),
                Err(er) => {
                    // thrown out of the JavaScript checker, not collected
                    self.crashed.get_or_insert(PcodeError { kind: ErrorKind::Internal, ..er });
                    return;
                }
            }
        }
        let mut si = Vec::new();
        for (ri, (row, rpos)) in tab.rows.iter().enumerate() {
            if row.len() != tab.outs.len() {
                self.e(format!("table {name}: the row has {} values, the table {} columns", row.len(), tab.outs.len()), rpos);
            }
            si.push(row.iter().enumerate().map(|(i, x)| x * scales.get(i).copied().unwrap_or(1.0)).collect::<Vec<f64>>());
            if ri > 0 && !(row[0] > tab.rows[ri - 1].0[0]) {
                self.e(format!("table {name}: keys rise strictly from row to row"), rpos);
            }
        }
        let min_rows = if tab.mode == TableMode::Linear { 2 } else { 1 };
        if tab.rows.len() < min_rows {
            self.e(format!("table {name}: too few rows"), &tab.pos);
        }
        self.tables[ti].si = si;
    }
}

/// Check parsed files together (they may call one another).
pub(crate) fn check(files: Vec<File>, exprs: Vec<Expr>) -> Result<Checked, Vec<PcodeError>> {
    let n = exprs.len();
    let c = Checker {
        exprs,
        ann: vec![Ann::default(); n],
        errors: Vec::new(),
        crashed: None,
        fns: Vec::new(),
        tables: Vec::new(),
        records: Vec::new(),
        consts: Vec::new(),
        data: Vec::new(),
        choices: Vec::new(),
        const_ty: Vec::new(),
        names: HashMap::new(),
        modules: Vec::new(),
        mod_items: Vec::new(),
        order: Vec::new(),
        scopes: Vec::new(),
        cur: None,
        cur_states: Vec::new(),
        touched: HashSet::new(),
    };
    c.run(files)
}
