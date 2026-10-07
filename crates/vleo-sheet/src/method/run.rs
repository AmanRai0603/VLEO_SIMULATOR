//! The method language: running: the interpreter.
//!
//! A method is read once into the form it runs in ([`Compiled`]): every name
//! it uses is found then, in the scopes the text gives it, and becomes the
//! place its value is kept — a slot, an input, a kernel constant — and every
//! function it calls becomes the function itself. Running it then looks
//! nothing up by name. A sweep runs a method thousands of times, and looking
//! each name up as text on every pass was most of what an answer cost.
//!
//! What it computes, and in what order, is exactly what the text says: the
//! same functions on the same values, so the answer is the same to the bit as
//! the translation's, and every refusal and every fault is the same, with the
//! same words, raised at the same point — a name with no value, say, only when
//! the line that reads it runs.

use super::*;

/// What a method gave for one set of inputs.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// The answer, in the SI unit of the node's quantity.
    Answer(f64),
    Refused {
        line: usize,
        reason: String,
    },
}

enum Flow {
    Next,
    Done(Outcome),
}

enum Val {
    Num(f64),
    Bool(bool),
}

/// Where a name's value is, found once when the method is read.
enum Place {
    /// A value the method named, in its slot.
    Slot(usize),
    /// One of the method's inputs, by its place among them.
    Input(usize),
    /// A constant of the kernel's.
    Constant(f64),
    /// Nothing: refused, by the name, when the line that reads it runs.
    Nowhere(String),
}

/// A row of `interp`'s table, found once.
enum Row {
    /// Written out in place: one of the method's tables.
    Table(usize),
    /// A list by name.
    List(usize, String),
    /// Neither.
    Not,
}

/// A function, found once.
enum Func {
    Plain1(fn(f64) -> f64),
    Plain2(fn(f64, f64) -> f64),
    Checked1(fn(f64, u32) -> Result<f64, rt::MethodError>),
    Checked2(fn(f64, f64, u32) -> Result<f64, rt::MethodError>),
    Kernel(&'static KernelFn),
    Unknown(String),
}

/// An expression, as it runs.
struct Node {
    line: usize,
    kind: Kind,
}

enum Kind {
    Num(f64),
    Bool(bool),
    Var(Place),
    Neg(Box<Node>),
    Not(Box<Node>),
    And(Box<Node>, Box<Node>),
    Or(Box<Node>, Box<Node>),
    Bin(BinOp, Box<Node>, Box<Node>),
    Interp(Box<Node>, Row, Row),
    /// `len` of a list, or its refusal when not given a list's name.
    Len(Option<(usize, String)>),
    Call(Func, Vec<Node>),
    /// A list written out where a number belongs.
    Table,
    Index(usize, String, Box<Node>),
}

/// A statement, as it runs.
enum Step {
    /// A value named, or named again: `let`, a `const` number.
    Name(usize, Node, usize),
    /// A list a `const` writes out.
    List(usize, usize),
    /// `set`: the slot it sets, if the name has one where it stands.
    Set(Option<usize>, Node, usize),
    If(Vec<(Node, Vec<Step>)>, Option<Vec<Step>>),
    For {
        slot: usize,
        first: i64,
        last: i64,
        body: Vec<Step>,
        line: usize,
    },
    Each {
        slot: usize,
        list: usize,
        name: String,
        body: Vec<Step>,
        line: usize,
    },
    While {
        cond: Node,
        max: i64,
        body: Vec<Step>,
        line: usize,
    },
    Refuse(String, usize),
    Return(Node, usize),
    Publish(String, Node, usize),
}

/// A method, read once into the form it runs in, for inputs of these names.
pub struct Compiled {
    inputs: Vec<String>,
    body: Vec<Step>,
    slots: usize,
    /// The method's tables: the lists its `const`s write out and the tables
    /// written straight into `interp`.
    tables: Vec<Vec<f64>>,
    /// How many names of lists the method uses.
    lists: usize,
}

/// What reading a method into its running form keeps track of.
struct Reader<'p> {
    inputs: &'p [String],
    /// The names in scope where the reader stands, innermost last.
    scopes: Vec<Vec<(&'p str, usize)>>,
    slots: usize,
    tables: Vec<Vec<f64>>,
    lists: Vec<&'p str>,
}

impl<'p> Reader<'p> {
    fn place(&self, name: &str) -> Place {
        for s in self.scopes.iter().rev() {
            if let Some((_, k)) = s.iter().find(|(n, _)| *n == name) {
                return Place::Slot(*k);
            }
        }
        if let Some(k) = self.inputs.iter().position(|n| n == name) {
            return Place::Input(k);
        }
        if let Some((v, _)) = kernel_constant(name) {
            return Place::Constant(v);
        }
        Place::Nowhere(name.to_string())
    }
    /// The slot a name takes where the reader stands: its own, if the
    /// innermost scope already names it, as a map's insert would.
    fn name(&mut self, name: &'p str) -> usize {
        let s = self.scopes.last_mut().expect("a scope");
        if let Some((_, k)) = s.iter().find(|(n, _)| *n == name) {
            return *k;
        }
        let k = self.slots;
        self.slots += 1;
        s.push((name, k));
        k
    }
    fn settable(&self, name: &str) -> Option<usize> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.iter().find(|(n, _)| *n == name).map(|(_, k)| *k))
    }
    /// A list's place by its name: one for every name, filled when the
    /// `const` that writes it out runs.
    fn list(&mut self, name: &'p str) -> usize {
        match self.lists.iter().position(|n| *n == name) {
            Some(k) => k,
            None => {
                self.lists.push(name);
                self.lists.len() - 1
            }
        }
    }
    fn table(&mut self, si: &[f64]) -> usize {
        self.tables.push(si.to_vec());
        self.tables.len() - 1
    }
    fn row(&mut self, e: &'p Expr) -> Row {
        match e {
            Expr::Table { si, .. } => Row::Table(self.table(si)),
            Expr::Var { name, .. } => Row::List(self.list(name), name.clone()),
            _ => Row::Not,
        }
    }
    fn expr(&mut self, e: &'p Expr) -> Node {
        let line = e.line();
        let kind = match e {
            Expr::Num { si, .. } => Kind::Num(*si),
            Expr::Bool { value, .. } => Kind::Bool(*value),
            Expr::Var { name, .. } => Kind::Var(self.place(name)),
            Expr::Neg(x) => Kind::Neg(Box::new(self.expr(x))),
            Expr::Not(x) => Kind::Not(Box::new(self.expr(x))),
            Expr::Table { .. } => Kind::Table,
            Expr::Index { list, index, .. } => {
                let k = self.list(list);
                Kind::Index(k, list.clone(), Box::new(self.expr(index)))
            }
            Expr::Bin { op, l, r, .. } => {
                let (l, r) = (Box::new(self.expr(l)), Box::new(self.expr(r)));
                match op {
                    BinOp::And => Kind::And(l, r),
                    BinOp::Or => Kind::Or(l, r),
                    _ => Kind::Bin(*op, l, r),
                }
            }
            Expr::Call { name, args, .. } if name == "interp" => {
                let x = Box::new(self.expr(&args[0]));
                Kind::Interp(x, self.row(&args[1]), self.row(&args[2]))
            }
            Expr::Call { name, args, .. } if name == "len" => match args.first() {
                Some(Expr::Var { name: l, .. }) => Kind::Len(Some((self.list(l), l.clone()))),
                _ => Kind::Len(None),
            },
            Expr::Call { name, args, .. } => {
                let args = args.iter().map(|a| self.expr(a)).collect();
                // The language's own functions first: no kernel function
                // shares a name with one.
                let f = match implementation(name) {
                    Some(Impl::Plain1(f, _)) => Func::Plain1(f),
                    Some(Impl::Plain2(f, _)) => Func::Plain2(f),
                    Some(Impl::Checked1(f, _)) => Func::Checked1(f),
                    Some(Impl::Checked2(f, _)) => Func::Checked2(f),
                    None => match kernel_function(name) {
                        Some(k) => Func::Kernel(k),
                        None => Func::Unknown(name.clone()),
                    },
                };
                Kind::Call(f, args)
            }
        };
        Node { line, kind }
    }
    fn block(&mut self, body: &'p [Stmt]) -> Vec<Step> {
        body.iter().map(|s| self.stmt(s)).collect()
    }
    /// A block in a scope of its own.
    fn scoped(&mut self, body: &'p [Stmt]) -> Vec<Step> {
        self.scopes.push(Vec::new());
        let b = self.block(body);
        self.scopes.pop();
        b
    }
    fn stmt(&mut self, s: &'p Stmt) -> Step {
        match s {
            Stmt::Const {
                name,
                expr: Expr::Table { si, .. },
                ..
            } => {
                let t = self.table(si);
                Step::List(self.list(name), t)
            }
            Stmt::Let {
                name, expr, line, ..
            }
            | Stmt::Const { name, expr, line } => {
                let e = self.expr(expr);
                Step::Name(self.name(name), e, *line)
            }
            Stmt::Set { name, expr, line } => {
                let e = self.expr(expr);
                Step::Set(self.settable(name), e, *line)
            }
            Stmt::If {
                arms, otherwise, ..
            } => {
                let arms = arms
                    .iter()
                    .map(|(c, body)| {
                        let c = self.expr(c);
                        (c, self.scoped(body))
                    })
                    .collect();
                let otherwise = otherwise.as_ref().map(|b| self.scoped(b));
                Step::If(arms, otherwise)
            }
            Stmt::For {
                var,
                first,
                last,
                body,
                line,
            } => {
                // The loop's name and its body share one scope a pass.
                self.scopes.push(Vec::new());
                let slot = self.name(var);
                let body = self.block(body);
                self.scopes.pop();
                Step::For {
                    slot,
                    first: *first,
                    last: *last,
                    body,
                    line: *line,
                }
            }
            Stmt::Each {
                var,
                list,
                body,
                line,
            } => {
                let l = self.list(list);
                self.scopes.push(Vec::new());
                let slot = self.name(var);
                let body = self.block(body);
                self.scopes.pop();
                Step::Each {
                    slot,
                    list: l,
                    name: list.clone(),
                    body,
                    line: *line,
                }
            }
            Stmt::While {
                cond,
                max,
                body,
                line,
            } => {
                let cond = self.expr(cond);
                Step::While {
                    cond,
                    max: *max,
                    body: self.scoped(body),
                    line: *line,
                }
            }
            Stmt::Refuse { reason, line } => Step::Refuse(reason.clone(), *line),
            Stmt::Return { expr, line } => Step::Return(self.expr(expr), *line),
            Stmt::Publish { name, expr, line } => {
                Step::Publish(name.clone(), self.expr(expr), *line)
            }
        }
    }
}

struct Machine<'c> {
    c: &'c Compiled,
    slots: Vec<f64>,
    inputs: &'c [f64],
    steps: u64,
    /// The members published so far, by symbol.
    published: Vec<(String, f64)>,
    /// Each list's table, once the `const` that writes it out has run.
    lists: Vec<Option<usize>>,
}

impl<'c> Machine<'c> {
    fn get(&self, p: &Place, line: usize) -> Result<f64, Diag> {
        match p {
            Place::Slot(k) => Ok(self.slots[*k]),
            Place::Input(k) => Ok(self.inputs[*k]),
            Place::Constant(v) => Ok(*v),
            Place::Nowhere(name) => Err(Diag::err(line, format!("«{name}» has no value"))),
        }
    }
    fn list(&self, k: usize, name: &str, line: usize) -> Result<&'c [f64], Diag> {
        let c = self.c;
        self.lists[k]
            .map(|t| c.tables[t].as_slice())
            .ok_or_else(|| Diag::err(line, format!("«{name}» is not a list")))
    }
    /// A row of `interp`'s table, lent: a loop calls `interp` thousands of
    /// times on the same table.
    fn row(&self, r: &Row, line: usize) -> Result<&'c [f64], Diag> {
        match r {
            Row::Table(t) => Ok(self.c.tables[*t].as_slice()),
            Row::List(k, name) => self.list(*k, name, line),
            Row::Not => Err(Diag::err(line, "interp needs its table")),
        }
    }
    fn num(&mut self, e: &Node) -> Result<f64, Diag> {
        match self.eval(e)? {
            Val::Num(v) => Ok(v),
            Val::Bool(_) => Err(Diag::err(e.line, "a condition where a number belongs")),
        }
    }
    fn cond(&mut self, e: &Node) -> Result<bool, Diag> {
        match self.eval(e)? {
            Val::Bool(b) => Ok(b),
            Val::Num(_) => Err(Diag::err(e.line, "a number where a condition belongs")),
        }
    }
    fn eval(&mut self, e: &Node) -> Result<Val, Diag> {
        let line = e.line;
        let v = match &e.kind {
            Kind::Num(si) => *si,
            Kind::Bool(value) => return Ok(Val::Bool(*value)),
            Kind::Var(p) => self.get(p, line)?,
            Kind::Neg(x) => -self.num(x)?,
            Kind::Not(x) => return Ok(Val::Bool(!self.cond(x)?)),
            Kind::Table => {
                return Err(Diag::err(
                    line,
                    "a list written out stands in a const, or inside interp(…)",
                ))
            }
            Kind::Index(k, name, index) => {
                let i = self.num(index)?;
                rt::at(self.list(*k, name, line)?, i, line as u32).map_err(rt_diag)?
            }
            Kind::And(l, r) => return Ok(Val::Bool(self.cond(l)? && self.cond(r)?)),
            Kind::Or(l, r) => return Ok(Val::Bool(self.cond(l)? || self.cond(r)?)),
            Kind::Bin(op, l, r) => {
                let (a, b) = (self.num(l)?, self.num(r)?);
                match op {
                    BinOp::Add => a + b,
                    BinOp::Sub => a - b,
                    BinOp::Mul => a * b,
                    BinOp::Div => rt::div(a, b, line as u32).map_err(rt_diag)?,
                    BinOp::Pow => rt::pow(a, b),
                    BinOp::Lt => return Ok(Val::Bool(a < b)),
                    BinOp::Le => return Ok(Val::Bool(a <= b)),
                    BinOp::Gt => return Ok(Val::Bool(a > b)),
                    BinOp::Ge => return Ok(Val::Bool(a >= b)),
                    BinOp::Eq => return Ok(Val::Bool(a == b)),
                    BinOp::Ne => return Ok(Val::Bool(a != b)),
                    BinOp::And | BinOp::Or => unreachable!("read as And and Or"),
                }
            }
            Kind::Interp(x, xs, ys) => {
                let x = self.num(x)?;
                let (xs, ys) = (self.row(xs, line)?, self.row(ys, line)?);
                pmath::interp(x, xs, ys)
            }
            Kind::Len(l) => match l {
                Some((k, name)) => self.list(*k, name, line)?.len() as f64,
                None => return Err(Diag::err(line, "len takes the name of a list")),
            },
            Kind::Call(f, args) => self.call(f, args, line)?,
        };
        // Finiteness is checked where a value is named or returned — see
        // `rt::fin` — exactly as the translated code checks it.
        Ok(Val::Num(v))
    }
    fn call(&mut self, f: &Func, args: &[Node], line: usize) -> Result<f64, Diag> {
        // The arguments, on the stack when there are few, as there almost
        // always are; every one evaluated, in order, before the call.
        let mut few = [0.0f64; 8];
        let mut many = Vec::new();
        let a: &[f64] = if args.len() <= few.len() {
            for (slot, e) in few.iter_mut().zip(args) {
                *slot = self.num(e)?;
            }
            &few[..args.len()]
        } else {
            for e in args {
                many.push(self.num(e)?);
            }
            &many
        };
        let l = line as u32;
        match f {
            Func::Plain1(f) => Ok(f(a[0])),
            Func::Plain2(f) => Ok(f(a[0], a[1])),
            Func::Checked1(f) => f(a[0], l).map_err(rt_diag),
            Func::Checked2(f) => f(a[0], a[1], l).map_err(rt_diag),
            Func::Kernel(k) => {
                let v = (k.eval)(a);
                if v.is_nan() && !k.refuses.is_empty() {
                    return Err(Diag::err(line, k.refuses));
                }
                Ok(v)
            }
            Func::Unknown(name) => Err(Diag::err(line, format!("«{name}» is not a function"))),
        }
    }
    fn block(&mut self, body: &[Step]) -> Result<Flow, Diag> {
        for s in body {
            if let Flow::Done(o) = self.step(s)? {
                return Ok(Flow::Done(o));
            }
        }
        Ok(Flow::Next)
    }
    fn pass(&mut self, line: usize) -> Result<(), Diag> {
        self.steps += 1;
        if self.steps > MAX_STEPS {
            return Err(Diag::err(line, "too many loop steps"));
        }
        Ok(())
    }
    fn step(&mut self, s: &Step) -> Result<Flow, Diag> {
        match s {
            Step::List(k, t) => self.lists[*k] = Some(*t),
            Step::Name(k, e, line) => {
                self.slots[*k] = rt::fin(self.num(e)?, *line as u32).map_err(rt_diag)?;
            }
            Step::Set(k, e, line) => {
                let v = rt::fin(self.num(e)?, *line as u32).map_err(rt_diag)?;
                if let Some(k) = k {
                    self.slots[*k] = v;
                }
            }
            Step::If(arms, otherwise) => {
                for (cond, body) in arms {
                    if self.cond(cond)? {
                        return self.block(body);
                    }
                }
                if let Some(body) = otherwise {
                    return self.block(body);
                }
            }
            Step::For {
                slot,
                first,
                last,
                body,
                line,
            } => {
                for i in *first..=*last {
                    self.pass(*line)?;
                    self.slots[*slot] = i as f64;
                    if let Flow::Done(o) = self.block(body)? {
                        return Ok(Flow::Done(o));
                    }
                }
            }
            Step::Each {
                slot,
                list,
                name,
                body,
                line,
            } => {
                let entries = self.list(*list, name, *line)?;
                for &v in entries {
                    self.pass(*line)?;
                    self.slots[*slot] = v;
                    if let Flow::Done(o) = self.block(body)? {
                        return Ok(Flow::Done(o));
                    }
                }
            }
            Step::While {
                cond,
                max,
                body,
                line,
            } => {
                let mut passes = 0i64;
                while self.cond(cond)? {
                    if passes == *max {
                        return Ok(Flow::Done(Outcome::Refused {
                            line: *line,
                            reason: unsettled(*line, *max),
                        }));
                    }
                    passes += 1;
                    self.pass(*line)?;
                    if let Flow::Done(o) = self.block(body)? {
                        return Ok(Flow::Done(o));
                    }
                }
            }
            Step::Refuse(reason, line) => {
                return Ok(Flow::Done(Outcome::Refused {
                    line: *line,
                    reason: reason.clone(),
                }))
            }
            Step::Return(e, line) => {
                let v = rt::fin(self.num(e)?, *line as u32).map_err(rt_diag)?;
                return Ok(Flow::Done(Outcome::Answer(v)));
            }
            Step::Publish(name, e, line) => {
                let v = rt::fin(self.num(e)?, *line as u32).map_err(rt_diag)?;
                self.published.push((name.clone(), v));
            }
        }
        Ok(Flow::Next)
    }
}

impl Compiled {
    /// `p`, read for inputs of these names, in this order.
    pub fn new(p: &Program, inputs: &[String]) -> Compiled {
        let mut r = Reader {
            inputs,
            scopes: vec![Vec::new()],
            slots: 0,
            tables: Vec::new(),
            lists: Vec::new(),
        };
        let body = r.block(&p.body);
        Compiled {
            inputs: inputs.to_vec(),
            body,
            slots: r.slots,
            tables: r.tables,
            lists: r.lists.len(),
        }
    }

    /// Run on one set of inputs, each in the SI unit of its quantity and in
    /// the order the method was read for: the answer, and the members it
    /// published beside it, by symbol, in the order it published them (empty
    /// for a refusal and for a node with one answer).
    pub fn run_all(&self, inputs: &[f64]) -> Result<(Outcome, Vec<(String, f64)>), Diag> {
        // The door: an input that is not a number is refused before the
        // first line, as the translated method refuses it, never carried into
        // the arithmetic to fault there.
        if let Some(k) = inputs.iter().position(|v| !v.is_finite()) {
            return Ok((
                Outcome::Refused {
                    line: 0,
                    reason: format!("the input «{}» is not a finite number", self.inputs[k]),
                },
                Vec::new(),
            ));
        }
        let mut m = Machine {
            c: self,
            slots: vec![0.0; self.slots],
            inputs,
            steps: 0,
            published: Vec::new(),
            lists: vec![None; self.lists],
        };
        match m.block(&self.body)? {
            Flow::Done(o @ Outcome::Answer(_)) => Ok((o, m.published)),
            Flow::Done(o) => Ok((o, Vec::new())),
            Flow::Next => Err(Diag::err(0, "the method ended without return or refuse")),
        }
    }
}

/// Why a loop that did not settle refuses — the same words in the translation.
pub(crate) fn unsettled(line: usize, max: i64) -> String {
    format!("the loop on line {line} did not settle within {max} passes")
}

/// Run a checked method on one set of inputs, each in the SI unit of its
/// quantity. An error is a fault in the method or its inputs — a square root
/// of a negative number, say — and never a refusal: only the method's own
/// `refuse` refuses.
pub fn run(p: &Program, inputs: &[(String, f64)]) -> Result<Outcome, Diag> {
    run_all(p, inputs).map(|(o, _)| o)
}

/// The same, with the members the method published beside its answer, by
/// symbol, in the order it published them. Empty for a refusal and for a node
/// with one answer.
pub fn run_all(
    p: &Program,
    inputs: &[(String, f64)],
) -> Result<(Outcome, Vec<(String, f64)>), Diag> {
    let names: Vec<String> = inputs.iter().map(|(n, _)| n.clone()).collect();
    let values: Vec<f64> = inputs.iter().map(|(_, v)| *v).collect();
    Compiled::new(p, &names).run_all(&values)
}
