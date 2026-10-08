//! The parsed program: expressions in one arena (an expression is its index), statements, the
//! declarations of a file, and the types as written and as the checker resolves them.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

use crate::error::Pos;
use crate::units::{dim_text, Dim};
use std::rc::Rc;

pub(crate) type ExprId = usize;

/// A type as it was written: `real[kg m^2]`, `vec3[m/s]`, `int[7]`, a record's name.
#[derive(Clone, Debug)]
pub(crate) enum TypeDecl {
    Int,
    Bool,
    /// a random stream (the toolbox's)
    Stream,
    Real { unit: String },
    /// `cap`: a named capacity (`real[1][NR]`), the const that states the length
    Arr { n: usize, of: Box<TypeDecl>, cap: Option<(String, Pos)> },
    /// a buffer, as long as the caller's array (`real[1][*]`): only an inout input of a fn is one
    Buf { of: Box<TypeDecl>, pos: Pos },
    Rec { name: String, pos: Pos },
}

impl TypeDecl {
    /// The unit of the innermost element, if it is real.
    pub(crate) fn elem_unit(&self) -> Option<&str> {
        match self {
            TypeDecl::Arr { of, .. } | TypeDecl::Buf { of, .. } => of.elem_unit(),
            TypeDecl::Real { unit } => Some(unit),
            _ => None,
        }
    }
}

/// A type as the checker knows it: a real carries its dimension.
#[derive(Clone, Debug, PartialEq)]
pub enum Ty {
    Int,
    Bool,
    Real(Dim),
    Arr(usize, Box<Ty>),
    /// a buffer: as long as the caller's array (an inout input of a fn), its elements numbers
    Buf(Box<Ty>),
    Rec(String),
    Tuple(Vec<Ty>),
    /// a choice, by its name: one of its options, numbered from 0
    Choice(String),
    /// a random stream (the toolbox's: adcs-sim-core rng.rs), held as six numbers
    Stream,
    /// a name in quotes: a stream's
    Str,
}

impl Ty {
    /// The type as the checker writes it in a message: `real[m/s]`, `real[1][3]`, `int`.
    pub fn text(&self) -> String {
        match self {
            Ty::Int => "int".into(),
            Ty::Bool => "bool".into(),
            Ty::Real(d) => format!("real[{}]", dim_text(d)),
            Ty::Arr(n, of) => format!("{}[{}]", of.text(), n),
            Ty::Buf(of) => format!("{}[*]", of.text()),
            Ty::Rec(name) | Ty::Choice(name) => name.clone(),
            Ty::Stream => "stream".into(),
            Ty::Str => "a name".into(),
            Ty::Tuple(items) => format!("({})", items.iter().map(Ty::text).collect::<Vec<_>>().join(", ")),
        }
    }
}

pub(crate) fn type_text(t: Option<&Ty>) -> String {
    t.map_or_else(|| "?".to_string(), Ty::text)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl BinOp {
    pub(crate) fn from_text(s: &str) -> Option<BinOp> {
        Some(match s {
            "+" => BinOp::Add,
            "-" => BinOp::Sub,
            "*" => BinOp::Mul,
            "/" => BinOp::Div,
            "^" => BinOp::Pow,
            "==" => BinOp::Eq,
            "!=" => BinOp::Ne,
            "<" => BinOp::Lt,
            "<=" => BinOp::Le,
            ">" => BinOp::Gt,
            ">=" => BinOp::Ge,
            "and" => BinOp::And,
            "or" => BinOp::Or,
            _ => return None,
        })
    }
    pub(crate) fn text(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Pow => "^",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::And => "and",
            BinOp::Or => "or",
        }
    }
    pub(crate) fn is_cmp(self) -> bool {
        matches!(self, BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ExprKind {
    /// A number; `unit` is the text in brackets after it, if any.
    Num { v: f64, is_int: bool, unit: Option<String> },
    Bool(bool),
    Var(String),
    Arr { items: Vec<ExprId>, unit: Option<String> },
    Index { a: ExprId, i: ExprId },
    Field { a: ExprId, f: String },
    Not(ExprId),
    Neg(ExprId),
    Ifx { c: ExprId, a: ExprId, b: ExprId },
    Bin { op: BinOp, a: ExprId, b: ExprId },
    /// A call; `f` is the name as written (`norm`, `orbit.period`).
    Call { f: String, args: Vec<ExprId> },
    /// A name in quotes (a stream's: `stream(seed, "gyro")`).
    Str(String),
}

#[derive(Clone, Debug)]
pub(crate) struct Expr {
    pub kind: ExprKind,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub(crate) struct Param {
    pub name: String,
    /// an input handed by reference (`img: inout real[1][4096]`): the caller's variable is changed in place
    pub inout: bool,
    pub decl: TypeDecl,
    pub ty: Ty,
    pub range: Option<(ExprId, ExprId)>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub(crate) enum StmtKind {
    Let { names: Vec<String>, decl: Option<TypeDecl>, e: ExprId },
    State { name: String, decl: TypeDecl, e: ExprId },
    If { arms: Vec<(ExprId, Vec<Stmt>)>, els: Option<Vec<Stmt>> },
    For { v: String, a: ExprId, b: ExprId, body: Vec<Stmt> },
    Settle { n: ExprId, c: ExprId, body: Vec<Stmt>, els: Option<Vec<Stmt>> },
    Set { targets: Vec<ExprId>, e: ExprId },
}

#[derive(Clone, Debug)]
pub(crate) struct Stmt {
    pub kind: StmtKind,
    pub pos: Pos,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FnKind {
    Fn,
    Proc,
}

#[derive(Clone, Debug)]
pub(crate) struct Func {
    pub kind: FnKind,
    pub name: String,
    pub module: String,
    pub params: Vec<Param>,
    pub outs: Vec<Param>,
    pub body: Rc<Vec<Stmt>>,
    pub doc: Vec<String>,
    pub pos: Pos,
    /// the state a proc keeps: its name and the constant it starts at
    pub states: Vec<(String, ExprId)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableMode {
    Step,
    Linear,
}

#[derive(Clone, Debug)]
pub(crate) struct Table {
    pub name: String,
    pub module: String,
    pub key: Param,
    pub outs: Vec<Param>,
    pub mode: TableMode,
    pub rows: Vec<(Vec<f64>, Pos)>,
    /// every row in SI
    pub si: Vec<Vec<f64>>,
    /// its `##` lines
    pub doc: Vec<String>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub(crate) struct Record {
    pub name: String,
    pub module: String,
    pub fields: Vec<Param>,
    /// its `##` lines
    pub doc: Vec<String>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub(crate) struct Const {
    pub name: String,
    pub module: String,
    pub e: ExprId,
    /// its `##` lines
    pub doc: Vec<String>,
    pub pos: Pos,
}

/// A data table: a 1-D or 2-D array of reals or ints whose values the design writes, row by row.
#[derive(Clone, Debug)]
pub(crate) struct Data {
    pub name: String,
    pub module: String,
    pub decl: TypeDecl,
    pub ty: Ty,
    pub rows: Vec<(Vec<f64>, Pos)>,
    /// every value in SI, the outer index first (None where the checker refused the table)
    pub si: Option<Vec<f64>>,
    /// its `##` lines
    pub doc: Vec<String>,
    pub pos: Pos,
}

/// A choice: its options, numbered from 0 in the translations.
#[derive(Clone, Debug)]
pub(crate) struct Choice {
    pub name: String,
    pub module: String,
    pub options: Vec<(String, Pos)>,
    /// its `##` lines
    pub doc: Vec<String>,
    pub pos: Pos,
}

/// A top-level declaration as parsed.
#[derive(Clone, Debug)]
pub(crate) enum Item {
    Func(Func),
    Table(Table),
    Record(Record),
    Const(Const),
    Data(Data),
    Choice(Choice),
}

impl Item {
    pub(crate) fn name(&self) -> &str {
        match self {
            Item::Func(f) => &f.name,
            Item::Table(t) => &t.name,
            Item::Record(r) => &r.name,
            Item::Const(c) => &c.name,
            Item::Data(d) => &d.name,
            Item::Choice(c) => &c.name,
        }
    }
    pub(crate) fn pos(&self) -> &Pos {
        match self {
            Item::Func(f) => &f.pos,
            Item::Table(t) => &t.pos,
            Item::Record(r) => &r.pos,
            Item::Const(c) => &c.pos,
            Item::Data(d) => &d.pos,
            Item::Choice(c) => &c.pos,
        }
    }
    pub(crate) fn set_module(&mut self, m: &str) {
        match self {
            Item::Func(f) => f.module = m.into(),
            Item::Table(t) => t.module = m.into(),
            Item::Record(r) => r.module = m.into(),
            Item::Const(c) => c.module = m.into(),
            Item::Data(d) => d.module = m.into(),
            Item::Choice(c) => c.module = m.into(),
        }
    }
}

/// One parsed file.
#[derive(Clone, Debug)]
pub(crate) struct File {
    pub module: Option<String>,
    pub doc: Vec<String>,
    pub uses: Vec<String>,
    pub items: Vec<Item>,
}

/// Every expression of the statements, each once, an expression before what it holds.
pub(crate) fn visit_stmts(stmts: &[Stmt], exprs: &[Expr], f: &mut dyn FnMut(ExprId)) {
    for s in stmts {
        match &s.kind {
            StmtKind::Let { e, .. } | StmtKind::State { e, .. } => visit_expr(*e, exprs, f),
            StmtKind::Set { targets, e } => {
                targets.iter().for_each(|t| visit_expr(*t, exprs, f));
                visit_expr(*e, exprs, f);
            }
            StmtKind::If { arms, els } => {
                for (c, body) in arms {
                    visit_expr(*c, exprs, f);
                    visit_stmts(body, exprs, f);
                }
                if let Some(els) = els {
                    visit_stmts(els, exprs, f);
                }
            }
            StmtKind::For { a, b, body, .. } => {
                visit_expr(*a, exprs, f);
                visit_expr(*b, exprs, f);
                visit_stmts(body, exprs, f);
            }
            StmtKind::Settle { n, c, body, els } => {
                visit_expr(*n, exprs, f);
                visit_expr(*c, exprs, f);
                visit_stmts(body, exprs, f);
                if let Some(els) = els {
                    visit_stmts(els, exprs, f);
                }
            }
        }
    }
}

/// An expression and every expression it holds, in the order they are written.
pub(crate) fn visit_expr(e: ExprId, exprs: &[Expr], f: &mut dyn FnMut(ExprId)) {
    f(e);
    match &exprs[e].kind {
        ExprKind::Num { .. } | ExprKind::Bool(_) | ExprKind::Var(_) | ExprKind::Str(_) => {}
        ExprKind::Arr { items, .. } => items.iter().for_each(|x| visit_expr(*x, exprs, f)),
        ExprKind::Index { a, i } => {
            visit_expr(*a, exprs, f);
            visit_expr(*i, exprs, f);
        }
        ExprKind::Field { a, .. } | ExprKind::Not(a) | ExprKind::Neg(a) => visit_expr(*a, exprs, f),
        ExprKind::Ifx { c, a, b } => {
            visit_expr(*c, exprs, f);
            visit_expr(*a, exprs, f);
            visit_expr(*b, exprs, f);
        }
        ExprKind::Bin { a, b, .. } => {
            visit_expr(*a, exprs, f);
            visit_expr(*b, exprs, f);
        }
        ExprKind::Call { args, .. } => args.iter().for_each(|x| visit_expr(*x, exprs, f)),
    }
}
