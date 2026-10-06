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

pub(super) fn to_matlab(i: &Interp, pkg: Option<&str>) -> Result<Files, String> {
    let pkg = pkg.filter(|p| !p.is_empty()).unwrap_or("asils.physics").to_string();
    let mut g = Gen { b: Base::new(i), pkg, tmp: 0 };
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
            Ty::Real(_) | Ty::Int => "0".into(),
            Ty::Bool => "false".into(),
            Ty::Arr(n, of) => match &**of {
                Ty::Arr(m, _) => format!("zeros({n}, {m})"),
                _ => format!("zeros({n}, 1)"),
            },
            Ty::Rec(name) => {
                let module = self.b.record(name).map_or("undefined", |r| r.module.as_str());
                format!("{}.{module}.{name}_zero()", self.pkg)
            }
            Ty::Tuple(_) => "undefined".into(),
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
            Ty::Bool | Ty::Int => js_string(v),
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
            Ty::Arr(_, of) => matches!(**of, Ty::Arr(..)),
            _ => {
                self.b.fail("cannot read properties of undefined (reading 'k')");
                false
            }
        }
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
            ExprKind::Num { .. } | ExprKind::Bool(_) | ExprKind::Var(_) => false,
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
            ExprKind::Var(name) => match ann.var {
                VarKind::Pi => "pi".into(),
                VarKind::Const(ci) => {
                    let k = &c.consts[ci];
                    let v = self.b.const_of(k.e);
                    let t = self.b.ty_of(k.e);
                    self.value_lit_m(&v, t)
                }
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
                if let Some((aa, ai)) = self.matrix_pair(*a) {
                    let m = self.ex(aa);
                    let r = self.idx(ai);
                    return format!("{m}({r}, {})", self.idx(*i));
                }
                if self.rows_of(*a) {
                    let m = self.ex(*a);
                    return format!("({m}({}, :)).'", self.idx(*i));
                }
                let m = self.ex(*a);
                format!("{m}({})", self.idx(*i))
            }
            ExprKind::Field { a, f } => format!("{}.{f}", self.ex(*a)),
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
            Shape::Mv => format!("{RTP}.mv({}, {})", self.ex(a), self.ex(b)),
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
                "sqrt" | "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "exp" | "log" | "log10" | "floor" | "ceil" | "round" => {
                    return format!("{f}({})", at(0))
                }
                "abs" => return format!("{RTP}.fabs({})", at(0)),
                "sign" => return format!("sign({})", at(0)),
                "atan2" => return format!("atan2({}, {})", at(0), at(1)),
                "hypot" => return format!("sqrt({0}*{0} + {1}*{1})", at(0), at(1)),
                "fmod" => return format!("rem({}, {})", at(0), at(1)),
                "pow" => return format!("({})^({})", at(0), at(1)),
                "min" | "max" => {
                    let Some((first, rest)) = a.split_first() else {
                        return self.b.fail("Reduce of empty array with no initial value");
                    };
                    return rest.iter().fold(first.clone(), |acc, x| format!("{RTP}.f{f}({acc}, {x})"));
                }
                "clamp" => return format!("{RTP}.clamp({}, {}, {})", at(0), at(1), at(2)),
                "real" => return at(0).to_string(),
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
                        _ => "undefined".into(),
                    }
                }
                "dot" => return format!("{RTP}.dot_({}, {})", at(0), at(1)),
                "cross" => return format!("{RTP}.cross_({}, {})", at(0), at(1)),
                "norm" => return format!("{RTP}.norm_({})", at(0)),
                "unit" => return format!("{RTP}.unit_({})", at(0)),
                "transpose" => return format!("({}).'", at(0)),
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

    fn stmts(&mut self, body: &[Stmt], ind: usize) -> String {
        body.iter().map(|s| self.stmt(s, ind)).collect()
    }
    fn stmt(&mut self, s: &Stmt, ind: usize) -> String {
        let ii = "    ".repeat(ind);
        match &s.kind {
            StmtKind::Let { names, e, .. } => {
                if names.len() > 1 {
                    return format!("{ii}[{}] = {};\n", names.join(", "), self.ex(*e));
                }
                format!("{ii}{} = {};\n", names[0], self.ex(*e))
            }
            StmtKind::State { .. } => String::new(),
            StmtKind::Set { targets, e } => {
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
                        let mut outs: Vec<&str> = f.outs.iter().map(|o| o.name.as_str()).collect();
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
                    ItemRef::Const(_) => {}
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
            let mut at = 0;
            let mut args = Vec::new();
            for p in &f.params {
                args.push(self.read_arg(&p.ty, at));
                at += self.b.flat(&p.ty);
            }
            let outs: Vec<String> = (0..f.outs.len()).map(|i| format!("o{}", i + 1)).collect();
            let ys: Vec<String> = f.outs.iter().zip(&outs).map(|(o, v)| self.flat_out(&o.ty, v)).collect();
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
                args.push(self.read_arg(&p.ty, at));
                at += self.b.flat(&p.ty);
            }
            let outs: Vec<String> = (0..f.outs.len()).map(|i| format!("o{}", i + 1)).collect();
            let ys: Vec<String> = f.outs.iter().zip(&outs).map(|(o, v)| self.flat_out(&o.ty, v)).collect();
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

    fn read_arg(&self, t: &Ty, at: usize) -> String {
        let n = self.b.flat(t);
        if let Ty::Rec(name) = t {
            let mut k = 0;
            let parts: Vec<String> = match self.b.record(name) {
                Some(r) => r
                    .fields
                    .iter()
                    .map(|f| {
                        let s = format!("'{}', {{{}}}", f.name, self.read_arg(&f.ty, at + k));
                        k += self.b.flat(&f.ty);
                        s
                    })
                    .collect(),
                None => Vec::new(),
            };
            return format!("struct({})", parts.join(", "));
        }
        let sl = if n == 1 { format!("x({})", at + 1) } else { format!("x({}:{})", at + 1, at + n) };
        match t {
            Ty::Bool => format!("({sl} ~= 0)"),
            Ty::Arr(n, of) => match &**of {
                Ty::Arr(m, _) => format!("reshape({sl}, {m}, {n}).'"),
                _ => format!("reshape({sl}, {n}, 1)"),
            },
            _ => sl,
        }
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
            Ty::Arr(..) => format!("reshape({v}, [], 1)"),
            _ => format!("double({v})"),
        }
    }
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
    add("dot_", "a, b", "s", "sum of products, left to right.", "    s = a(1)*b(1);\n    for i = 2:numel(a), s = s + a(i)*b(i); end\n");
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
    f
}
