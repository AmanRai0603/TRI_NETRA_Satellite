//! The lexer and the parser: a file's text to its declarations, as design/js/pcode.js reads it,
//! token for token, with the same messages at the same line and column.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

use crate::ast::*;
use crate::error::{ErrorKind, PcodeError, Pos};
use crate::jsfmt::json_str;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum T {
    Doc,
    Nl,
    Num,
    Kw,
    Id,
    Str,
    Op,
    Eof,
}

impl T {
    fn name(self) -> &'static str {
        match self {
            T::Doc => "doc",
            T::Nl => "nl",
            T::Num => "num",
            T::Kw => "kw",
            T::Id => "id",
            T::Str => "str",
            T::Op => "op",
            T::Eof => "eof",
        }
    }
}

#[derive(Clone, Debug)]
struct Tok {
    t: T,
    v: String,
    line: u32,
    col: u32,
    is_int: bool,
}

const KEYWORDS: &[&str] = &[
    "module", "use", "const", "fn", "proc", "table", "record", "end", "let", "state", "if", "then", "elif", "else", "for", "in",
    "settle", "until", "and", "or", "not", "true", "false", "step", "linear",
];
const OPS: &[&str] = &["->", "..", "==", "!=", "<=", ">=", "+", "-", "*", "/", "^", "(", ")", "[", "]", ",", ":", "=", "<", ">", ".", "|"];

fn utf16_len(s: &[char]) -> u32 {
    s.iter().map(|c| c.len_utf16() as u32).sum()
}

/// The length of the number at the start of `s` (`/^(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?/`).
fn number_len(s: &[char]) -> usize {
    let digits = |from: usize| s[from..].iter().take_while(|c| c.is_ascii_digit()).count();
    let mut n = digits(0);
    if n > 0 {
        if s.get(n) == Some(&'.') {
            n += 1 + digits(n + 1);
        }
    } else {
        n = 1 + digits(1); // `.` and at least one digit (the caller saw it)
    }
    if matches!(s.get(n), Some('e') | Some('E')) {
        let mut m = n + 1;
        if matches!(s.get(m), Some('+') | Some('-')) {
            m += 1;
        }
        let d = digits(m);
        if d > 0 {
            n = m + d;
        }
    }
    n
}

fn lex(src: &str, file: &Rc<str>) -> Result<Vec<Tok>, PcodeError> {
    let s: Vec<char> = src.chars().collect();
    let mut toks = Vec::new();
    let (mut i, mut line, mut col, mut depth) = (0usize, 1u32, 1u32, 0i32);
    let push = |toks: &mut Vec<Tok>, t: T, v: String, line: u32, col: u32, is_int: bool| toks.push(Tok { t, v, line, col, is_int });
    let err = |m: String, line: u32, col: u32| PcodeError::new(ErrorKind::Lex, m, &Pos { file: Some(file.clone()), line, col });
    while i < s.len() {
        let ch = s[i];
        if ch == '#' {
            let doc = s.get(i + 1) == Some(&'#');
            let mut j = i;
            while j < s.len() && s[j] != '\n' {
                j += 1;
            }
            if doc {
                let text: String = s[(i + 2).min(j)..j].iter().collect();
                push(&mut toks, T::Doc, text.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}').to_string(), line, col, false);
            }
            col += utf16_len(&s[i..j]);
            i = j;
            continue;
        }
        if ch == '\n' {
            if depth == 0 {
                push(&mut toks, T::Nl, "\n".into(), line, col, false);
            }
            i += 1;
            line += 1;
            col = 1;
            continue;
        }
        if ch == ' ' || ch == '\t' || ch == '\r' {
            i += 1;
            col += 1;
            continue;
        }
        if ch == '\\' && s.get(i + 1) == Some(&'\n') {
            i += 2;
            line += 1;
            col = 1;
            continue;
        }
        if ch.is_ascii_digit() || (ch == '.' && s.get(i + 1).is_some_and(|c| c.is_ascii_digit())) {
            let n = number_len(&s[i..]);
            let mut text: String = s[i..i + n].iter().collect();
            // `1..` is the number 1 and the range's `..`
            if s.get(i + n) == Some(&'.') && s.get(i + n + 1) == Some(&'.') && text.ends_with('.') {
                text.pop();
            }
            let len = text.chars().count();
            let is_int = !text.contains(['.', 'e', 'E']);
            push(&mut toks, T::Num, text, line, col, is_int);
            i += len;
            col += len as u32;
            continue;
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
            let n = s[i..].iter().take_while(|c| c.is_ascii_alphanumeric() || **c == '_').count();
            let word: String = s[i..i + n].iter().collect();
            let t = if KEYWORDS.contains(&word.as_str()) { T::Kw } else { T::Id };
            push(&mut toks, t, word, line, col, false);
            i += n;
            col += n as u32;
            continue;
        }
        if ch == '"' {
            let mut j = i + 1;
            while j < s.len() && s[j] != '"' && s[j] != '\n' {
                j += 1;
            }
            if s.get(j) != Some(&'"') {
                return Err(err("a string is not closed on its line".into(), line, col));
            }
            push(&mut toks, T::Str, s[i + 1..j].iter().collect(), line, col, false);
            col += utf16_len(&s[i..=j]);
            i = j + 1;
            continue;
        }
        let op = OPS.iter().find(|o| {
            let oc: Vec<char> = o.chars().collect();
            s.len() >= i + oc.len() && s[i..i + oc.len()] == oc[..]
        });
        let Some(op) = op else {
            // JavaScript reads one UTF-16 unit: a character beyond the first plane is its high surrogate
            let shown = if ch.len_utf16() == 2 {
                let mut b = [0u16; 2];
                ch.encode_utf16(&mut b);
                format!("\"\\u{:04x}\"", b[0])
            } else {
                json_str(&ch.to_string())
            };
            return Err(err(format!("a character the language does not use: {shown}"), line, col));
        };
        if *op == "(" || *op == "[" {
            depth += 1;
        }
        if *op == ")" || *op == "]" {
            depth = (depth - 1).max(0);
        }
        push(&mut toks, T::Op, op.to_string(), line, col, false);
        i += op.len();
        col += op.len() as u32;
    }
    push(&mut toks, T::Nl, "\n".into(), line, col, false);
    push(&mut toks, T::Eof, String::new(), line, col, false);
    Ok(toks)
}

struct Parser<'a> {
    toks: Vec<Tok>,
    p: usize,
    file: Rc<str>,
    exprs: &'a mut Vec<Expr>,
}

type R<T> = Result<T, PcodeError>;

fn num_value(text: &str) -> f64 {
    text.parse::<f64>().unwrap_or(f64::NAN)
}

impl Parser<'_> {
    fn peek(&self, k: usize) -> &Tok {
        &self.toks[(self.p + k).min(self.toks.len() - 1)]
    }
    fn pos_of(&self, t: &Tok) -> Pos {
        Pos { file: Some(self.file.clone()), line: t.line, col: t.col }
    }
    fn err_at<X>(&self, msg: String, t: &Tok) -> R<X> {
        Err(PcodeError::new(ErrorKind::Parse, msg, &self.pos_of(t)))
    }
    fn err<X>(&self, msg: String) -> R<X> {
        self.err_at(msg, self.peek(0))
    }
    fn is(&self, t: T, v: Option<&str>) -> bool {
        let k = self.peek(0);
        k.t == t && v.is_none_or(|v| k.v == v)
    }
    fn found(&self) -> String {
        let v = &self.peek(0).v;
        if v == "\n" {
            "the end of the line".into()
        } else {
            json_str(v)
        }
    }
    fn eat(&mut self, t: T, v: Option<&str>) -> R<Tok> {
        if !self.is(t, v) {
            return self.err(format!("expected {}, found {}", v.unwrap_or(t.name()), self.found()));
        }
        self.p += 1;
        Ok(self.toks[self.p - 1].clone())
    }
    fn eat_op(&mut self, v: &str) -> R<Tok> {
        self.eat(T::Op, Some(v))
    }
    fn eat_kw(&mut self, v: &str) -> R<Tok> {
        self.eat(T::Kw, Some(v))
    }
    fn opt(&mut self, t: T, v: &str) -> Option<Tok> {
        if self.is(t, Some(v)) {
            self.p += 1;
            Some(self.toks[self.p - 1].clone())
        } else {
            None
        }
    }
    fn skip_nl(&mut self) {
        while self.is(T::Nl, None) {
            self.p += 1;
        }
    }
    fn end_line(&mut self) -> R<()> {
        if !self.is(T::Eof, None) {
            self.eat(T::Nl, None)?;
        }
        Ok(())
    }
    fn add(&mut self, kind: ExprKind, pos: Pos) -> ExprId {
        self.exprs.push(Expr { kind, pos });
        self.exprs.len() - 1
    }

    /// inside [ ... ]: the unit's text rebuilt from its tokens
    fn unit_text(&mut self) -> R<String> {
        self.eat_op("[")?;
        let mut parts = Vec::new();
        while !self.is(T::Op, Some("]")) {
            if self.is(T::Eof, None) || self.is(T::Nl, None) {
                return self.err("a unit is not closed with ]".into());
            }
            parts.push(self.toks[self.p].v.clone());
            self.p += 1;
        }
        self.eat_op("]")?;
        Ok(parts.join(" ").replace(" ^ ", "^").replace("^ - ", "^-").replace(" / ", "/").replace(" * ", " "))
    }
    fn typ(&mut self) -> R<TypeDecl> {
        let t = self.peek(0).clone();
        if self.opt(T::Id, "int").is_some() {
            return self.arr_suffix(TypeDecl::Int);
        }
        if self.opt(T::Id, "bool").is_some() {
            return self.arr_suffix(TypeDecl::Bool);
        }
        if self.opt(T::Id, "stream").is_some() {
            return self.arr_suffix(TypeDecl::Stream); // a random stream (the toolbox's)
        }
        if self.opt(T::Id, "quat").is_some() {
            return self.arr_suffix(TypeDecl::Arr { n: 4, of: Box::new(TypeDecl::Real { unit: "1".into() }), cap: None });
        }
        for (name, n, mat) in [("vec3", 3, false), ("mat3", 3, true), ("vec4", 4, false), ("vec2", 2, false)] {
            if self.opt(T::Id, name).is_some() {
                let unit = if self.is(T::Op, Some("[")) { self.unit_text()? } else { "1".into() };
                let mut ty = TypeDecl::Arr { n, of: Box::new(TypeDecl::Real { unit }), cap: None };
                if mat {
                    ty = TypeDecl::Arr { n, of: Box::new(ty), cap: None };
                }
                return self.arr_suffix(ty);
            }
        }
        if self.opt(T::Id, "real").is_some() {
            let unit = if self.is(T::Op, Some("[")) { self.unit_text()? } else { "1".into() };
            return self.arr_suffix(TypeDecl::Real { unit });
        }
        if self.is(T::Id, None) {
            let name = self.eat(T::Id, None)?.v;
            return self.arr_suffix(TypeDecl::Rec { name, pos: self.pos_of(&t) });
        }
        self.err("expected a type (real[unit], int, bool, vec3[unit], mat3[unit], quat, or a record)".into())
    }
    fn arr_suffix(&mut self, mut ty: TypeDecl) -> R<TypeDecl> {
        while self.is(T::Op, Some("[")) {
            self.eat_op("[")?;
            // a named capacity: the length a whole-number const states (`const NR = 8`, `real[1][NR]`)
            if self.is(T::Id, None) {
                let c = self.eat(T::Id, None)?;
                self.eat_op("]")?;
                ty = TypeDecl::Arr { n: 1, of: Box::new(ty), cap: Some((c.v.clone(), self.pos_of(&c))) };
                continue;
            }
            // a buffer: as long as the caller's array (`img: inout real[1][*]`, len(img) its length), the outermost
            if self.is(T::Op, Some("*")) {
                let s = self.eat_op("*")?;
                self.eat_op("]")?;
                ty = TypeDecl::Buf { of: Box::new(ty), pos: self.pos_of(&s) };
                if self.is(T::Op, Some("[")) {
                    return self.err("a buffer [*] is the outermost: real[1][3][*], not real[1][*][3]".into());
                }
                continue;
            }
            let n = self.eat(T::Num, None)?;
            self.eat_op("]")?;
            let v = num_value(&n.v);
            if !n.is_int || v < 1.0 {
                return self.err_at("an array's length is a whole number from 1".into(), &n);
            }
            ty = TypeDecl::Arr { n: v as usize, of: Box::new(ty), cap: None };
        }
        Ok(ty)
    }
    fn param(&mut self) -> R<Param> {
        let t = self.eat(T::Id, None)?;
        self.eat_op(":")?;
        // an input handed by reference: `img: inout real[1][4096]` (the caller's variable is changed in place)
        let mut inout = false;
        if self.is(T::Id, Some("inout")) && self.peek(1).t == T::Id {
            self.p += 1;
            inout = true;
        }
        let decl = self.typ()?;
        let mut range = None;
        if self.opt(T::Kw, "in").is_some() {
            let lo = self.expr()?;
            self.eat_op("..")?;
            let hi = self.expr()?;
            range = Some((lo, hi));
        }
        Ok(Param { name: t.v.clone(), inout, decl, ty: Ty::Int, range, pos: self.pos_of(&t) })
    }
    fn outputs(&mut self) -> R<Vec<Param>> {
        if self.opt(T::Op, "(").is_some() {
            let mut outs = vec![self.param()?];
            while self.opt(T::Op, ",").is_some() {
                outs.push(self.param()?);
            }
            self.eat_op(")")?;
            return Ok(outs);
        }
        Ok(vec![self.param()?])
    }
    fn block(&mut self, stops: &[&str]) -> R<Vec<Stmt>> {
        let mut body = Vec::new();
        loop {
            self.skip_nl();
            if self.is(T::Eof, None) {
                return self.err(format!("the block is not closed (expected {})", stops.join(" or ")));
            }
            if self.is(T::Doc, None) {
                self.p += 1;
                continue;
            }
            if self.is(T::Kw, None) && stops.contains(&self.peek(0).v.as_str()) {
                return Ok(body);
            }
            body.push(self.stmt()?);
        }
    }
    fn stmt(&mut self) -> R<Stmt> {
        let t = self.peek(0).clone();
        let pos = self.pos_of(&t);
        if self.opt(T::Kw, "let").is_some() {
            let mut names = vec![self.eat(T::Id, None)?.v];
            let mut decl = None;
            if self.opt(T::Op, ":").is_some() {
                decl = Some(self.typ()?);
            }
            while self.opt(T::Op, ",").is_some() {
                names.push(self.eat(T::Id, None)?.v);
            }
            self.eat_op("=")?;
            let e = self.expr()?;
            self.end_line()?;
            return Ok(Stmt { kind: StmtKind::Let { names, decl, e }, pos });
        }
        if self.opt(T::Kw, "state").is_some() {
            let n = self.eat(T::Id, None)?;
            self.eat_op(":")?;
            let decl = self.typ()?;
            self.eat_op("=")?;
            let e = self.expr()?;
            self.end_line()?;
            return Ok(Stmt { kind: StmtKind::State { name: n.v, decl, e }, pos });
        }
        if self.opt(T::Kw, "if").is_some() {
            let mut arms = Vec::new();
            let c = self.expr()?;
            self.end_line()?;
            let body = self.block(&["elif", "else", "end"])?;
            arms.push((c, body));
            let mut els = None;
            loop {
                if self.opt(T::Kw, "elif").is_some() {
                    let c = self.expr()?;
                    self.end_line()?;
                    let body = self.block(&["elif", "else", "end"])?;
                    arms.push((c, body));
                    continue;
                }
                if self.opt(T::Kw, "else").is_some() {
                    self.end_line()?;
                    els = Some(self.block(&["end"])?);
                }
                self.eat_kw("end")?;
                self.end_line()?;
                break;
            }
            return Ok(Stmt { kind: StmtKind::If { arms, els }, pos });
        }
        if self.opt(T::Kw, "for").is_some() {
            let v = self.eat(T::Id, None)?;
            self.eat_kw("in")?;
            let a = self.expr()?;
            self.eat_op("..")?;
            let b = self.expr()?;
            self.end_line()?;
            let body = self.block(&["end"])?;
            self.eat_kw("end")?;
            self.end_line()?;
            return Ok(Stmt { kind: StmtKind::For { v: v.v, a, b, body }, pos });
        }
        if self.opt(T::Kw, "settle").is_some() {
            self.eat(T::Id, Some("max"))?;
            let n = self.expr()?;
            self.eat_kw("until")?;
            let c = self.expr()?;
            self.end_line()?;
            let body = self.block(&["else", "end"])?;
            let mut els = None;
            if self.opt(T::Kw, "else").is_some() {
                self.end_line()?;
                els = Some(self.block(&["end"])?);
            }
            self.eat_kw("end")?;
            self.end_line()?;
            return Ok(Stmt { kind: StmtKind::Settle { n, c, body, els }, pos });
        }
        // assignment: target[, target] = expr
        let mut targets = vec![self.lvalue()?];
        while self.opt(T::Op, ",").is_some() {
            targets.push(self.lvalue()?);
        }
        self.eat_op("=")?;
        let e = self.expr()?;
        self.end_line()?;
        Ok(Stmt { kind: StmtKind::Set { targets, e }, pos })
    }
    fn lvalue(&mut self) -> R<ExprId> {
        let t = self.eat(T::Id, None)?;
        let mut lv = self.add(ExprKind::Var(t.v.clone()), self.pos_of(&t));
        loop {
            if self.is(T::Op, Some("[")) {
                let b = self.eat_op("[")?;
                let i = self.expr()?;
                self.eat_op("]")?;
                lv = self.add(ExprKind::Index { a: lv, i }, self.pos_of(&b));
                continue;
            }
            if self.is(T::Op, Some(".")) {
                let b = self.eat_op(".")?;
                let f = self.eat(T::Id, None)?;
                lv = self.add(ExprKind::Field { a: lv, f: f.v }, self.pos_of(&b));
                continue;
            }
            return Ok(lv);
        }
    }

    // expressions, lowest precedence first
    fn expr(&mut self) -> R<ExprId> {
        if self.is(T::Kw, Some("if")) {
            let t = self.eat_kw("if")?;
            let c = self.expr()?;
            self.eat_kw("then")?;
            let a = self.expr()?;
            self.eat_kw("else")?;
            let b = self.expr()?;
            return Ok(self.add(ExprKind::Ifx { c, a, b }, self.pos_of(&t)));
        }
        self.orx()
    }
    fn bin_level(&mut self, ops: &[&str], next: fn(&mut Self) -> R<ExprId>) -> R<ExprId> {
        let mut a = next(self)?;
        loop {
            let t = self.peek(0).clone();
            if !((t.t == T::Op || t.t == T::Kw) && ops.contains(&t.v.as_str())) {
                return Ok(a);
            }
            self.p += 1;
            let b = next(self)?;
            let op = BinOp::from_text(&t.v).unwrap();
            a = self.add(ExprKind::Bin { op, a, b }, self.pos_of(&t));
        }
    }
    fn orx(&mut self) -> R<ExprId> {
        self.bin_level(&["or"], Self::andx)
    }
    fn andx(&mut self) -> R<ExprId> {
        self.bin_level(&["and"], Self::notx)
    }
    fn addx(&mut self) -> R<ExprId> {
        self.bin_level(&["+", "-"], Self::mulx)
    }
    fn mulx(&mut self) -> R<ExprId> {
        self.bin_level(&["*", "/"], Self::unx)
    }
    fn notx(&mut self) -> R<ExprId> {
        if self.is(T::Kw, Some("not")) {
            let t = self.eat_kw("not")?;
            let a = self.notx()?;
            return Ok(self.add(ExprKind::Not(a), self.pos_of(&t)));
        }
        self.cmpx()
    }
    fn cmpx(&mut self) -> R<ExprId> {
        const CMP: [&str; 6] = ["==", "!=", "<", "<=", ">", ">="];
        let a = self.addx()?;
        let t = self.peek(0).clone();
        if !(t.t == T::Op && CMP.contains(&t.v.as_str())) {
            return Ok(a);
        }
        self.p += 1;
        let b = self.addx()?;
        if CMP.iter().any(|o| self.is(T::Op, Some(o))) {
            return self.err("comparisons do not chain: write (a < b) and (b < c)".into());
        }
        let op = BinOp::from_text(&t.v).unwrap();
        Ok(self.add(ExprKind::Bin { op, a, b }, self.pos_of(&t)))
    }
    fn unx(&mut self) -> R<ExprId> {
        if self.is(T::Op, Some("-")) {
            let t = self.eat_op("-")?;
            let a = self.unx()?;
            return Ok(self.add(ExprKind::Neg(a), self.pos_of(&t)));
        }
        if self.is(T::Op, Some("+")) {
            self.eat_op("+")?;
            return self.unx();
        }
        self.powx()
    }
    fn powx(&mut self) -> R<ExprId> {
        let a = self.postfix()?;
        if self.is(T::Op, Some("^")) {
            let t = self.eat_op("^")?;
            let b = self.unx()?;
            return Ok(self.add(ExprKind::Bin { op: BinOp::Pow, a, b }, self.pos_of(&t)));
        }
        Ok(a)
    }
    fn postfix(&mut self) -> R<ExprId> {
        let mut a = self.atom()?;
        // a qualified name `mod.name` waiting for its `(`
        let mut qual: Option<String> = None;
        loop {
            let is_var = matches!(self.exprs[a].kind, ExprKind::Var(_));
            if self.is(T::Op, Some("(")) && (is_var || qual.is_some()) {
                let t = self.eat_op("(")?;
                let mut args = Vec::new();
                if !self.is(T::Op, Some(")")) {
                    args.push(self.expr()?);
                    while self.opt(T::Op, ",").is_some() {
                        args.push(self.expr()?);
                    }
                }
                self.eat_op(")")?;
                let f = match qual.take() {
                    Some(q) => q,
                    None => match &self.exprs[a].kind {
                        ExprKind::Var(n) => n.clone(),
                        _ => unreachable!(),
                    },
                };
                a = self.add(ExprKind::Call { f, args }, self.pos_of(&t));
                continue;
            }
            if self.is(T::Op, Some("[")) {
                let t = self.eat_op("[")?;
                let i = self.expr()?;
                self.eat_op("]")?;
                a = self.add(ExprKind::Index { a, i }, self.pos_of(&t));
                continue;
            }
            if self.is(T::Op, Some(".")) && self.peek(1).t == T::Id {
                let t = self.eat_op(".")?;
                let f = self.eat(T::Id, None)?;
                if qual.is_none() && is_var && self.is(T::Op, Some("(")) {
                    if let ExprKind::Var(m) = &self.exprs[a].kind {
                        qual = Some(format!("{m}.{}", f.v));
                    }
                    continue;
                }
                a = self.add(ExprKind::Field { a, f: f.v }, self.pos_of(&t));
                continue;
            }
            return Ok(a);
        }
    }
    fn atom(&mut self) -> R<ExprId> {
        let t = self.peek(0).clone();
        let pos = self.pos_of(&t);
        if self.is(T::Num, None) {
            self.p += 1;
            let mut unit = None;
            if self.is(T::Op, Some("[")) && !(self.peek(1).t == T::Num && self.peek(2).t == T::Op && self.peek(2).v == "]") {
                unit = Some(self.unit_text()?);
            }
            let is_int = t.is_int && unit.is_none();
            return Ok(self.add(ExprKind::Num { v: num_value(&t.v), is_int, unit }, pos));
        }
        if self.opt(T::Kw, "true").is_some() {
            return Ok(self.add(ExprKind::Bool(true), pos));
        }
        if self.is(T::Str, None) {
            // a name in quotes: a stream's (stream(seed, "gyro"))
            self.p += 1;
            return Ok(self.add(ExprKind::Str(t.v.clone()), pos));
        }
        if self.opt(T::Kw, "false").is_some() {
            return Ok(self.add(ExprKind::Bool(false), pos));
        }
        if self.is(T::Id, None) {
            self.p += 1;
            return Ok(self.add(ExprKind::Var(t.v.clone()), pos));
        }
        if self.opt(T::Op, "(").is_some() {
            let e = self.expr()?;
            self.eat_op(")")?;
            return Ok(e);
        }
        if self.opt(T::Op, "|").is_some() {
            let e = self.expr()?;
            self.eat_op("|")?;
            return Ok(self.add(ExprKind::Call { f: "abs".into(), args: vec![e] }, pos));
        }
        if self.opt(T::Op, "[").is_some() {
            let mut items = Vec::new();
            if !self.is(T::Op, Some("]")) {
                items.push(self.expr()?);
                while self.opt(T::Op, ",").is_some() {
                    items.push(self.expr()?);
                }
            }
            self.eat_op("]")?;
            let mut unit = None;
            if self.is(T::Op, Some("[")) && self.peek(1).t != T::Num {
                unit = Some(self.unit_text()?);
            }
            return Ok(self.add(ExprKind::Arr { items, unit }, pos));
        }
        let found = if t.v == "\n" { "the end of the line".to_string() } else { json_str(&t.v) };
        self.err(format!("expected a value, found {found}"))
    }

    fn file(&mut self) -> R<File> {
        let mut prog = File { module: None, doc: Vec::new(), uses: Vec::new(), items: Vec::new() };
        let mut docs: Vec<String> = Vec::new();
        loop {
            self.skip_nl();
            if self.is(T::Eof, None) {
                break;
            }
            if self.is(T::Doc, None) {
                docs.push(self.eat(T::Doc, None)?.v);
                continue;
            }
            let t = self.peek(0).clone();
            let pos = self.pos_of(&t);
            let doc = std::mem::take(&mut docs);
            if self.opt(T::Kw, "module").is_some() {
                prog.module = Some(self.eat(T::Id, None)?.v);
                self.end_line()?;
                prog.doc = doc;
                continue;
            }
            if self.opt(T::Kw, "use").is_some() {
                prog.uses.push(self.eat(T::Id, None)?.v);
                self.end_line()?;
                continue;
            }
            if self.opt(T::Kw, "const").is_some() {
                let n = self.eat(T::Id, None)?;
                self.eat_op("=")?;
                let e = self.expr()?;
                self.end_line()?;
                prog.items.push(Item::Const(Const { name: n.v, module: String::new(), e, doc, pos }));
                continue;
            }
            if self.is(T::Kw, Some("fn")) || self.is(T::Kw, Some("proc")) {
                let kind = if self.toks[self.p].v == "fn" { FnKind::Fn } else { FnKind::Proc };
                self.p += 1;
                let n = self.eat(T::Id, None)?;
                self.eat_op("(")?;
                let mut params = Vec::new();
                if !self.is(T::Op, Some(")")) {
                    params.push(self.param()?);
                    while self.opt(T::Op, ",").is_some() {
                        params.push(self.param()?);
                    }
                }
                self.eat_op(")")?;
                self.eat_op("->")?;
                let outs = self.outputs()?;
                self.end_line()?;
                let body = self.block(&["end"])?;
                self.eat_kw("end")?;
                self.end_line()?;
                prog.items.push(Item::Func(Func {
                    kind,
                    name: n.v,
                    module: String::new(),
                    params,
                    outs,
                    body: Rc::new(body),
                    doc,
                    pos,
                    states: Vec::new(),
                }));
                continue;
            }
            if self.opt(T::Kw, "record").is_some() {
                let n = self.eat(T::Id, None)?;
                self.end_line()?;
                let mut fields = Vec::new();
                loop {
                    self.skip_nl();
                    if self.is(T::Doc, None) {
                        self.p += 1;
                        continue;
                    }
                    if self.opt(T::Kw, "end").is_some() {
                        break;
                    }
                    fields.push(self.param()?);
                    self.end_line()?;
                }
                self.end_line()?;
                prog.items.push(Item::Record(Record { name: n.v, module: String::new(), fields, doc, pos }));
                continue;
            }
            if self.opt(T::Kw, "table").is_some() {
                let n = self.eat(T::Id, None)?;
                self.eat_op("(")?;
                let key = self.param()?;
                self.eat_op(")")?;
                self.eat_op("->")?;
                let outs = self.outputs()?;
                let mut mode = TableMode::Linear;
                if self.opt(T::Kw, "step").is_some() {
                    mode = TableMode::Step;
                } else if self.opt(T::Kw, "linear").is_some() {
                    mode = TableMode::Linear;
                }
                self.end_line()?;
                let rows = self.num_rows()?;
                self.end_line()?;
                prog.items.push(Item::Table(Table {
                    name: n.v,
                    module: String::new(),
                    key,
                    outs,
                    mode,
                    rows,
                    si: Vec::new(),
                    doc,
                    pos,
                }));
                continue;
            }
            // a choice: `choice Kind = rw, fmr, cmg`, its options numbered from 0 in the translations
            if self.is(T::Id, Some("choice")) && self.peek(1).t == T::Id {
                self.p += 1;
                let n = self.eat(T::Id, None)?;
                self.eat_op("=")?;
                let o = self.eat(T::Id, None)?;
                let mut options = vec![(o.v.clone(), self.pos_of(&o))];
                while self.opt(T::Op, ",").is_some() {
                    let x = self.eat(T::Id, None)?;
                    options.push((x.v.clone(), self.pos_of(&x)));
                }
                self.end_line()?;
                prog.items.push(Item::Choice(Choice { name: n.v, module: String::new(), options, doc, pos }));
                continue;
            }
            // a data table (the design writes its values): `data NAME: real[unit][C][R]`, then its values row by row
            if self.is(T::Id, Some("data")) && self.peek(1).t == T::Id {
                self.p += 1;
                let n = self.eat(T::Id, None)?;
                self.eat_op(":")?;
                let decl = self.typ()?;
                self.end_line()?;
                let rows = self.num_rows()?;
                self.end_line()?;
                prog.items.push(Item::Data(Data { name: n.v, module: String::new(), decl, ty: Ty::Int, rows, si: None, doc, pos }));
                continue;
            }
            return self.err("expected module, use, const, data, choice, fn, proc, record or table at the top level".into());
        }
        Ok(prog)
    }
    /// the rows of numbers of a table or a data table, up to its `end`
    fn num_rows(&mut self) -> R<Vec<(Vec<f64>, Pos)>> {
        let mut rows = Vec::new();
        loop {
            self.skip_nl();
            if self.is(T::Doc, None) {
                self.p += 1;
                continue;
            }
            if self.opt(T::Kw, "end").is_some() {
                break;
            }
            let rt = self.peek(0).clone();
            let mut row = vec![self.table_num()?];
            while self.opt(T::Op, ",").is_some() {
                row.push(self.table_num()?);
            }
            rows.push((row, self.pos_of(&rt)));
            self.end_line()?;
        }
        Ok(rows)
    }
    fn table_num(&mut self) -> R<f64> {
        let neg = self.opt(T::Op, "-").is_some();
        let v = self.eat(T::Num, None)?;
        Ok((if neg { -1.0 } else { 1.0 }) * num_value(&v.v))
    }
}

/// One file's text to its declarations; its expressions go into `exprs`.
pub(crate) fn parse(src: &str, file: &str, exprs: &mut Vec<Expr>) -> Result<File, PcodeError> {
    let file: Rc<str> = Rc::from(file);
    let toks = lex(src, &file)?;
    let mut p = Parser { toks, p: 0, file, exprs };
    p.file()
}
