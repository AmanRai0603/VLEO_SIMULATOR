//! The method language: checking: names, dimensions, and that every path ends.

use super::*;

/// What a method is checked against: the node's inputs, by binding name, with
/// their dimensions, and the dimension of its answer.
#[derive(Clone, Debug)]
pub struct Signature {
    pub inputs: Vec<(String, Dim)>,
    pub output: Dim,
    /// The members the node publishes beside its answer, by symbol, in the
    /// order its sheet declares them. Empty for a node with one answer.
    pub publishes: Vec<(String, Dim)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Ty {
    Num(Dim),
    /// A bare `0` or a bare literal, which adopts the other side's dimension
    /// in a sum or comparison. Only zero does: `h < 0` is fine, `h < 5` is
    /// not, because 5 of what is exactly the question.
    Zero,
    Pure(f64),
    Bool,
}

impl Ty {
    fn dim(self) -> Option<Dim> {
        match self {
            Ty::Num(d) => Some(d),
            Ty::Zero | Ty::Pure(_) => Some(Dim::NONE),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Input,
    Kernel,
    Const,
    Let,
    Loop,
    /// A list written out with const: read by entry, by `for … in`, by `len`
    /// and by `interp` — never as one number.
    List,
}

struct Checker<'a> {
    scopes: Vec<BTreeMap<String, (Kind, Dim)>>,
    diags: Vec<Diag>,
    read: BTreeSet<String>,
    sig: &'a Signature,
    iterations: u64,
    /// Every list the method writes out, by name: its unit and its entries in
    /// SI. A list is written at the top level, so one name is one list.
    lists: BTreeMap<String, (Dim, Vec<f64>)>,
}

impl Checker<'_> {
    fn lookup(&self, name: &str) -> Option<(Kind, Dim)> {
        for s in self.scopes.iter().rev() {
            if let Some(v) = s.get(name) {
                return Some(*v);
            }
        }
        if let Some((_, d)) = kernel_constant(name) {
            return Some((Kind::Kernel, d));
        }
        None
    }
    fn err(&mut self, line: usize, msg: impl Into<String>) {
        self.diags.push(Diag::err(line, msg));
    }
    fn define(&mut self, name: &str, kind: Kind, dim: Dim, line: usize) {
        if RUST_UNRAW.contains(&name) || name.starts_with("__") || name == "_" {
            self.err(
                line,
                format!("«{name}» cannot be a name in a method — choose another"),
            );
            return;
        }
        if let Some((k, _)) = self.lookup(name) {
            let what = match k {
                Kind::Input => "an input of this node",
                Kind::Kernel => "a kernel constant",
                _ => "already defined",
            };
            self.err(line, format!("«{name}» is {what} — choose another name"));
            return;
        }
        self.scopes
            .last_mut()
            .expect("a scope")
            .insert(name.to_string(), (kind, dim));
    }

    /// Both sides of a sum or comparison, as one dimension.
    fn like(&mut self, a: Ty, b: Ty, line: usize, what: &str) -> Option<Dim> {
        match (a, b) {
            (Ty::Zero, Ty::Zero) => Some(Dim::NONE),
            (Ty::Zero, t) | (t, Ty::Zero) => t.dim(),
            _ => {
                let (Some(x), Some(y)) = (a.dim(), b.dim()) else {
                    self.err(line, format!("{what} needs two numbers"));
                    return None;
                };
                if x != y {
                    self.err(
                        line,
                        format!(
                            "{what} of unlike quantities: {} and {}",
                            dim_text(x),
                            dim_text(y)
                        ),
                    );
                    return None;
                }
                Some(x)
            }
        }
    }

    fn expr(&mut self, e: &Expr) -> Option<Ty> {
        match e {
            Expr::Num { si, dim, bare, .. } => Some(if *bare && *si == 0.0 {
                Ty::Zero
            } else if *bare {
                Ty::Pure(*si)
            } else {
                Ty::Num(*dim)
            }),
            Expr::Bool { .. } => Some(Ty::Bool),
            Expr::Var { name, line } => {
                match self.lookup(name) {
                    Some((Kind::List, _)) => {
                        self.err(
                            *line,
                            format!("«{name}» is a list — read one entry as {name}[i], or its length as len({name})"),
                        );
                        None
                    }
                    Some((kind, d)) => {
                        if kind == Kind::Input {
                            self.read.insert(name.clone());
                        }
                        Some(Ty::Num(d))
                    }
                    None => {
                        let hint = if function(name).is_some() {
                            " — it is a function; call it with brackets".to_string()
                        } else {
                            String::new()
                        };
                        self.err(
                        *line,
                        format!("«{name}» is not an input, a constant or a value made with let{hint}"),
                    );
                        None
                    }
                }
            }
            Expr::Neg(x) => match self.expr(x)? {
                Ty::Bool => {
                    self.err(e.line(), "a condition cannot be negated with - (use not)");
                    None
                }
                Ty::Pure(v) => Some(Ty::Pure(-v)),
                t => Some(t),
            },
            Expr::Not(x) => {
                if self.expr(x)? != Ty::Bool {
                    self.err(e.line(), "not applies to a condition");
                    return None;
                }
                Some(Ty::Bool)
            }
            Expr::Table { line, .. } => {
                self.err(
                    *line,
                    "a list written out stands in a const, or inside interp(…)",
                );
                None
            }
            Expr::Index { list, index, line } => self.index(list, index, *line),
            Expr::Bin { op, l, r, line } => self.binary(*op, l, r, *line),
            Expr::Call { name, args, line } => self.call(name, args, *line),
        }
    }

    /// The list a name stands for, or a finding saying why it does not.
    fn list(&mut self, name: &str, line: usize) -> Option<(Dim, Vec<f64>)> {
        if let Some(l) = self.lists.get(name) {
            return Some(l.clone());
        }
        let msg = if self.lookup(name).is_some() {
            format!("«{name}» is one value, not a list")
        } else {
            format!("«{name}» is not a list written out with const")
        };
        self.err(line, msg);
        None
    }

    /// `LIST[i]`: a pure-number index, counted from 1, that the checker holds
    /// to the list's length when it is written as a number.
    fn index(&mut self, name: &str, index: &Expr, line: usize) -> Option<Ty> {
        let (d, xs) = self.list(name, line)?;
        let t = self.expr(index)?;
        if t.dim() != Some(Dim::NONE) {
            self.err(
                line,
                format!("{name}[…] is read at a pure number, counted from 1 — not a condition or a quantity"),
            );
            return None;
        }
        let at = match t {
            Ty::Zero => Some(0.0),
            Ty::Pure(v) => Some(v),
            _ => None,
        };
        if let Some(v) = at {
            if v.fract() != 0.0 || v < 1.0 || v > xs.len() as f64 {
                self.err(
                    line,
                    format!(
                        "{name} has {} entries, counted from 1 — there is no entry {v}",
                        xs.len()
                    ),
                );
                return None;
            }
        }
        Some(Ty::Num(d))
    }

    fn binary(&mut self, op: BinOp, l: &Expr, r: &Expr, line: usize) -> Option<Ty> {
        if op == BinOp::Pow {
            let a = self.expr(l)?;
            return self.power(a, r, line);
        }
        let (a, b) = (self.expr(l), self.expr(r));
        let (a, b) = (a?, b?);
        match op {
            BinOp::And | BinOp::Or => {
                if a != Ty::Bool || b != Ty::Bool {
                    self.err(line, format!("{} joins two conditions", op.symbol()));
                    return None;
                }
                Some(Ty::Bool)
            }
            _ if op.is_compare() => {
                self.like(a, b, line, "a comparison")?;
                Some(Ty::Bool)
            }
            BinOp::Add | BinOp::Sub => {
                if let (Ty::Pure(x), Ty::Pure(y)) = (a, b) {
                    return Some(Ty::Pure(if op == BinOp::Add { x + y } else { x - y }));
                }
                let what = if op == BinOp::Add {
                    "a sum"
                } else {
                    "a difference"
                };
                self.like(a, b, line, what).map(Ty::Num)
            }
            BinOp::Mul | BinOp::Div => {
                if let (Ty::Pure(x), Ty::Pure(y)) = (a, b) {
                    return Some(Ty::Pure(if op == BinOp::Mul { x * y } else { x / y }));
                }
                let (Some(x), Some(y)) = (a.dim(), b.dim()) else {
                    self.err(line, format!("{} needs two numbers", op.symbol()));
                    return None;
                };
                Some(Ty::Num(if op == BinOp::Mul {
                    dim_mul(x, y)
                } else {
                    dim_div(x, y)
                }))
            }
            _ => unreachable!("every operator is handled above"),
        }
    }

    fn power(&mut self, base: Ty, exp: &Expr, line: usize) -> Option<Ty> {
        let Some(d) = base.dim() else {
            self.err(line, "^ raises a number");
            return None;
        };
        if let Some(p) = exp.literal() {
            self.expr(exp)?;
            if let Ty::Pure(b) = base {
                return Some(Ty::Pure(pmath::powf(b, p)));
            }
            if is_none(d) {
                return Some(Ty::Num(d));
            }
            // A dimensioned base needs a power that keeps every exponent whole:
            // m^2 ^ 0.5 is fine, m ^ 0.5 is not a quantity.
            for den in [1, 2, 3, 4, 6] {
                let num = p * den as f64;
                if num.fract() == 0.0 && num.abs() < 100.0 {
                    if let Some(r) = dim_pow(d, num as i32, den) {
                        return Some(Ty::Num(r));
                    }
                }
            }
            self.err(
                line,
                format!("{} to the power {p} is not a quantity", dim_text(d)),
            );
            return None;
        }
        let t = self.expr(exp)?;
        if !is_none(d) || t.dim() != Some(Dim::NONE) {
            self.err(
                line,
                "a computed power needs a pure number on both sides — a dimensioned value takes a written power, like r ^ 3",
            );
            return None;
        }
        Some(Ty::Num(Dim::NONE))
    }

    /// A kernel function: every argument in the unit its table names, the
    /// answer in the unit it gives.
    fn kernel_call(&mut self, k: &KernelFn, args: &[Expr], line: usize) -> Option<Ty> {
        if args.len() != k.args.len() {
            let names: Vec<&str> = k.args.iter().map(|a| a.0).collect();
            self.err(
                line,
                format!(
                    "{}({}) takes {} arguments, not {}",
                    k.name,
                    names.join(", "),
                    k.args.len(),
                    args.len()
                ),
            );
            return None;
        }
        let mut ok = true;
        // Every `~` argument in the unit of the first one given.
        let mut alike: Option<(Dim, &str)> = None;
        for (a, (an, au)) in args.iter().zip(k.args) {
            if *au == ANY {
                let t = self.expr(a)?;
                if matches!(t, Ty::Zero) {
                    continue;
                }
                let Some(d) = t.dim() else {
                    ok = false;
                    self.err(
                        line,
                        format!("{}: {an} is a condition, not a value", k.name),
                    );
                    continue;
                };
                match alike {
                    None => alike = Some((d, an)),
                    Some((first, by)) if first != d => {
                        ok = false;
                        self.err(
                            line,
                            format!(
                                "{}: {an} is in [{}] and {by} in [{}]; the kernel takes them in one unit",
                                k.name,
                                dim_text(d),
                                dim_text(first)
                            ),
                        );
                    }
                    Some(_) => {}
                }
                continue;
            }
            let want = parse_unit(au).map(|u| u.1).unwrap_or(Dim::NONE);
            match self.expr(a)? {
                Ty::Zero => {}
                Ty::Num(d) if d == want => {}
                Ty::Pure(_) if want == Dim::NONE => {}
                t => {
                    ok = false;
                    self.err(
                        line,
                        format!(
                            "{}: {an} is in [{}] here, and the kernel takes it in [{au}]",
                            k.name,
                            t.dim()
                                .map(dim_text)
                                .unwrap_or_else(|| "a condition".into())
                        ),
                    );
                }
            }
        }
        if !ok {
            return None;
        }
        let out = parse_unit(k.out).map(|u| u.1).unwrap_or(Dim::NONE);
        Some(Ty::Num(out))
    }

    fn call(&mut self, name: &str, args: &[Expr], line: usize) -> Option<Ty> {
        if let Some(k) = kernel_function(name) {
            return self.kernel_call(k, args, line);
        }
        let Some(f) = function(name) else {
            let hint = if self.lookup(name).is_some() {
                format!(" — «{name}» is a value, not a function")
            } else {
                " — the functions are listed in docs/PSEUDOCODE.md".to_string()
            };
            self.err(
                line,
                format!("«{name}» is not a function of the language{hint}"),
            );
            return None;
        };
        if args.len() != f.arity {
            self.err(
                line,
                format!(
                    "{name} takes {} argument{}, not {}",
                    f.arity,
                    if f.arity == 1 { "" } else { "s" },
                    args.len()
                ),
            );
            return None;
        }
        if f.rule == FnRule::Interp {
            return self.interp(args, line);
        }
        if f.rule == FnRule::Pow {
            let b = self.expr(&args[0])?;
            return self.power(b, &args[1], line);
        }
        if f.rule == FnRule::Len {
            let Expr::Var { name, .. } = &args[0] else {
                self.err(line, "len takes the name of a list");
                return None;
            };
            let (_, xs) = self.list(name, line)?;
            return Some(Ty::Pure(xs.len() as f64));
        }
        let mut tys = Vec::new();
        for a in args {
            tys.push(self.expr(a)?);
        }
        let dims: Vec<Dim> = tys.iter().map(|t| t.dim().unwrap_or(Dim::NONE)).collect();
        if tys.iter().any(|t| t.dim().is_none()) {
            self.err(line, format!("{name} takes numbers, not conditions"));
            return None;
        }
        match f.rule {
            FnRule::Pure => {
                if !is_none(dims[0]) {
                    self.err(
                        line,
                        format!(
                            "{name} of a {} value — it needs a pure number (divide by a reference first)",
                            dim_text(dims[0])
                        ),
                    );
                    return None;
                }
                Some(Ty::Num(Dim::NONE))
            }
            FnRule::Same => Some(Ty::Num(dims[0])),
            FnRule::SameTwo | FnRule::Ratio => {
                let d = self.like(tys[0], tys[1], line, name)?;
                Some(Ty::Num(if f.rule == FnRule::Ratio {
                    Dim::NONE
                } else {
                    d
                }))
            }
            FnRule::Sqrt | FnRule::Cbrt => {
                let den = if f.rule == FnRule::Sqrt { 2 } else { 3 };
                match dim_pow(dims[0], 1, den) {
                    Some(d) => Some(Ty::Num(d)),
                    None => {
                        self.err(
                            line,
                            format!("{name} of {} is not a quantity", dim_text(dims[0])),
                        );
                        None
                    }
                }
            }
            FnRule::Interp | FnRule::Pow | FnRule::Len => unreachable!("handled above"),
        }
    }

    /// A row of `interp`'s table: written out in place, or a list by name.
    fn row(&mut self, e: &Expr, line: usize) -> Option<(Dim, Vec<f64>)> {
        match e {
            Expr::Table { si, dim, .. } => Some((*dim, si.clone())),
            Expr::Var { name, .. } => self.list(name, line),
            _ => {
                self.err(
                    line,
                    "interp(x, XS, YS) — each row a list by name, or written out as [x1, x2, …] [unit]",
                );
                None
            }
        }
    }

    fn interp(&mut self, args: &[Expr], line: usize) -> Option<Ty> {
        let x = self.expr(&args[0])?;
        let (dx, xs) = self.row(&args[1], line)?;
        let (dy, ys) = self.row(&args[2], line)?;
        if xs.len() != ys.len() || xs.len() < 2 {
            self.err(
                line,
                format!(
                    "the table's two rows must be the same length, at least 2 ({} and {})",
                    xs.len(),
                    ys.len()
                ),
            );
            return None;
        }
        if xs.windows(2).any(|w| w[1] <= w[0]) {
            self.err(line, "the table's x row must rise strictly");
            return None;
        }
        self.like(x, Ty::Num(dx), line, "interp's x and its table")?;
        Some(Ty::Num(dy))
    }

    /// Check a block, and say whether every path through it ends in return
    /// or refuse.
    fn block(&mut self, body: &[Stmt]) -> bool {
        let mut ends = false;
        for s in body {
            if ends {
                self.err(
                    s.line(),
                    "this can never run — the line before it always ends the method",
                );
                break;
            }
            ends = self.stmt(s);
        }
        ends
    }

    fn stmt(&mut self, s: &Stmt) -> bool {
        match s {
            Stmt::Let {
                name,
                quantity,
                expr,
                line,
            } => {
                let t = self.expr(expr);
                let d = match t {
                    Some(Ty::Bool) => {
                        self.err(*line, "a value made with let is a number, not a condition");
                        None
                    }
                    Some(t) => t.dim(),
                    None => None,
                };
                if let (Some(q), Some(d)) = (quantity, d) {
                    match quantity_dim(q) {
                        None => self.err(*line, format!("«{q}» is not a quantity type")),
                        Some(qd) if qd != d => self.err(
                            *line,
                            format!(
                                "{name} is declared {q} ({}) but is {}",
                                dim_text(qd),
                                dim_text(d)
                            ),
                        ),
                        _ => {}
                    }
                }
                self.define(name, Kind::Let, d.unwrap_or(Dim::NONE), *line);
                false
            }
            Stmt::Const {
                name,
                expr: Expr::Table { si, dim, .. },
                line,
            } => {
                if self.scopes.len() > 1 {
                    self.err(
                        *line,
                        "a list is written at the method's top level, not inside if, for or while",
                    );
                }
                let before = self.diags.len();
                self.define(name, Kind::List, *dim, *line);
                if self.diags.len() == before {
                    self.lists.insert(name.clone(), (*dim, si.clone()));
                }
                false
            }
            Stmt::Const { name, expr, line } => {
                let d = self.expr(expr).and_then(|t| t.dim()).unwrap_or(Dim::NONE);
                self.define(name, Kind::Const, d, *line);
                false
            }
            Stmt::Set { name, expr, line } => {
                let t = self.expr(expr);
                match self.lookup(name) {
                    None => self.err(*line, format!("«{name}» has not been made with let yet")),
                    Some((Kind::Let, d)) => {
                        if let Some(t) = t {
                            self.like(Ty::Num(d), t, *line, "set")
                                .map(|_| ())
                                .unwrap_or(());
                        }
                    }
                    Some((k, _)) => {
                        let what = match k {
                            Kind::Input => "an input",
                            Kind::Loop => "the loop's own variable",
                            Kind::List => "a list",
                            _ => "a constant",
                        };
                        self.err(*line, format!("«{name}» is {what} and cannot be changed"));
                    }
                }
                false
            }
            Stmt::If {
                arms, otherwise, ..
            } => {
                let mut all = true;
                for (cond, body) in arms {
                    if let Some(t) = self.expr(cond) {
                        if t != Ty::Bool {
                            self.err(cond.line(), "if needs a condition, such as h < 0 [m]");
                        }
                    }
                    self.scopes.push(BTreeMap::new());
                    all &= self.block(body);
                    self.scopes.pop();
                }
                match otherwise {
                    Some(body) => {
                        self.scopes.push(BTreeMap::new());
                        all &= self.block(body);
                        self.scopes.pop();
                    }
                    None => all = false,
                }
                all
            }
            Stmt::For {
                var,
                first,
                last,
                body,
                line,
            } => {
                if last < first {
                    self.err(
                        *line,
                        format!("the loop runs from {first} to {last}, which is never"),
                    );
                }
                self.iterations = self
                    .iterations
                    .saturating_mul(1)
                    .saturating_add((last - first + 1).max(0) as u64);
                if self.iterations > MAX_STEPS {
                    self.err(
                        *line,
                        format!("more than {MAX_STEPS} loop steps — a method this long is a node to split"),
                    );
                }
                self.scopes.push(BTreeMap::new());
                self.define(var, Kind::Loop, Dim::NONE, *line);
                self.block(body);
                self.scopes.pop();
                false
            }
            Stmt::Each {
                var,
                list,
                body,
                line,
            } => {
                let d = match self.list(list, *line) {
                    Some((d, xs)) => {
                        self.iterations = self.iterations.saturating_add(xs.len() as u64);
                        d
                    }
                    None => Dim::NONE,
                };
                self.scopes.push(BTreeMap::new());
                self.define(var, Kind::Loop, d, *line);
                self.block(body);
                self.scopes.pop();
                false
            }
            Stmt::While {
                cond,
                max,
                body,
                line,
            } => {
                if let Some(t) = self.expr(cond) {
                    if t != Ty::Bool {
                        self.err(
                            cond.line(),
                            "while needs a condition, such as change > 1e-9 [m]",
                        );
                    }
                }
                if *max < 1 {
                    self.err(
                        *line,
                        format!("the loop may run at most {max} times, which is never"),
                    );
                }
                self.iterations = self.iterations.saturating_add((*max).max(0) as u64);
                if self.iterations > MAX_STEPS {
                    self.err(
                        *line,
                        format!("more than {MAX_STEPS} loop steps — a method this long is a node to split"),
                    );
                }
                self.scopes.push(BTreeMap::new());
                self.block(body);
                self.scopes.pop();
                false
            }
            Stmt::Refuse { reason, line } => {
                if reason.trim().is_empty() {
                    self.err(*line, "a refusal says why");
                }
                true
            }
            Stmt::Publish { name, expr, line } => {
                let want = self
                    .sig
                    .publishes
                    .iter()
                    .find(|(n, _)| n == name)
                    .map(|(_, d)| *d);
                match (self.expr(expr), want) {
                    (_, None) => {}
                    (Some(Ty::Bool), Some(_)) => {
                        self.err(*line, format!("«{name}» is a number, not a condition"))
                    }
                    (Some(Ty::Zero), Some(_)) | (None, Some(_)) => {}
                    (Some(Ty::Pure(_)), Some(w)) if is_none(w) => {}
                    (Some(t), Some(w)) => {
                        let d = t.dim().unwrap_or(Dim::NONE);
                        if d != w {
                            self.err(
                                *line,
                                format!(
                                    "«{name}» must be {} and this is {}",
                                    dim_text(w),
                                    dim_text(d)
                                ),
                            );
                        }
                    }
                }
                false
            }
            Stmt::Return { expr, line } => {
                match self.expr(expr) {
                    Some(Ty::Bool) => self.err(*line, "the answer is a number, not a condition"),
                    Some(t) => {
                        let want = self.sig.output;
                        match t {
                            Ty::Zero => {}
                            Ty::Pure(_) if is_none(want) => {}
                            _ => {
                                let d = t.dim().unwrap_or(Dim::NONE);
                                if d != want {
                                    self.err(
                                        *line,
                                        format!(
                                            "the answer must be {} and this is {}",
                                            dim_text(want),
                                            dim_text(d)
                                        ),
                                    );
                                }
                            }
                        }
                    }
                    None => {}
                }
                true
            }
        }
    }
}

/// The rules for a node that publishes several values: each member is
/// published once, at the method's top level, by a symbol the node declares,
/// and no path returns before every member is published — so an answer never
/// leaves with a member missing. A method may still refuse anywhere.
fn publishing(body: &[Stmt], sig: &Signature, diags: &mut Vec<Diag>) {
    fn nested(body: &[Stmt], diags: &mut Vec<Diag>) {
        for s in body {
            for inner in children(s) {
                for t in inner {
                    if let Stmt::Publish { line, .. } = t {
                        diags.push(Diag::err(
                            *line,
                            "publish stands at the method's top level, not inside if, for or while",
                        ));
                    }
                }
                nested(inner, diags);
            }
        }
    }
    fn children(s: &Stmt) -> Vec<&[Stmt]> {
        match s {
            Stmt::If {
                arms, otherwise, ..
            } => arms
                .iter()
                .map(|(_, b)| b.as_slice())
                .chain(otherwise.iter().map(|b| b.as_slice()))
                .collect(),
            Stmt::For { body, .. } | Stmt::Each { body, .. } | Stmt::While { body, .. } => {
                vec![body.as_slice()]
            }
            _ => Vec::new(),
        }
    }
    fn returns(s: &Stmt) -> bool {
        matches!(s, Stmt::Return { .. }) || children(s).into_iter().flatten().any(returns)
    }
    nested(body, diags);
    let mut done: BTreeSet<&str> = BTreeSet::new();
    for s in body {
        if let Stmt::Publish { name, line, .. } = s {
            if !sig.publishes.iter().any(|(n, _)| n == name) {
                diags.push(Diag::err(
                    *line,
                    if sig.publishes.is_empty() {
                        "this node publishes nothing beside its answer".to_string()
                    } else {
                        format!(
                            "the node publishes no member «{name}» — it publishes {}",
                            sig.publishes
                                .iter()
                                .map(|(n, _)| n.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    },
                ));
            } else if !done.insert(name.as_str()) {
                diags.push(Diag::err(*line, format!("«{name}» is published twice")));
            }
        }
        if returns(s) {
            let missing: Vec<&str> = sig
                .publishes
                .iter()
                .map(|(n, _)| n.as_str())
                .filter(|n| !done.contains(n))
                .collect();
            if !missing.is_empty() {
                diags.push(Diag::err(
                    s.line(),
                    format!(
                        "this returns before publishing {} — every answer carries every member",
                        missing.join(", ")
                    ),
                ));
                return;
            }
        }
    }
}

/// Check a parsed method against the node it belongs to. Errors and notes,
/// in line order; the method is sound when none is an error.
pub fn check(p: &Program, sig: &Signature) -> Vec<Diag> {
    let mut c = Checker {
        scopes: vec![BTreeMap::new()],
        diags: Vec::new(),
        read: BTreeSet::new(),
        sig,
        iterations: 0,
        lists: BTreeMap::new(),
    };
    for (name, d) in &sig.inputs {
        if kernel_constant(name).is_some() {
            c.err(
                0,
                format!("the input «{name}» has the name of a kernel constant"),
            );
        }
        c.scopes[0].insert(name.clone(), (Kind::Input, *d));
    }
    publishing(&p.body, sig, &mut c.diags);
    let ends = c.block(&p.body);
    if !ends {
        let line = p.body.last().map(|s| s.line()).unwrap_or(0);
        c.err(
            line,
            "some path reaches the end without return or refuse — every path must end in one",
        );
    }
    for (name, _) in &sig.inputs {
        if !c.read.contains(name) {
            c.diags.push(Diag::warn(
                0,
                format!("the input «{name}» is declared but the method never reads it"),
            ));
        }
    }
    c.diags.sort_by_key(|d| d.line);
    c.diags
}

/// Parse and check in one step: the program, or every error.
pub fn compile(src: &str, sig: &Signature) -> Result<Program, Vec<Diag>> {
    let p = parse(src).map_err(|d| vec![d])?;
    let diags = check(&p, sig);
    if diags.iter().any(|d| d.severity == Severity::Error) {
        return Err(diags);
    }
    Ok(p)
}
