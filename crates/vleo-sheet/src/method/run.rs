//! The method language: running: the interpreter.
//!
//! A method is read once into the form it runs in ([`Compiled`]): every name
//! it uses is found then, in the scopes the text gives it, and becomes the
//! place its value is kept — a slot, an input, a kernel constant — and every
//! function it calls becomes the function itself. Each expression and each
//! statement becomes a function of the values as they stand, built from the
//! functions of its parts, so running the method is calling them: nothing is
//! looked up by name and no tree is walked. A sweep runs a method thousands of
//! times, and that walking and looking up was most of what an answer cost.
//!
//! What it computes, and in what order, is exactly what the text says: the
//! same functions on the same values, so the answer is the same to the bit
//! every time, and every refusal and every fault is the same, with the same
//! words, raised at the same point — a name with no value, say, only when
//! the line that reads it runs, and a condition where a number belongs only
//! once the condition has been worked out.

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

/// The values a method's running form works on, as they stand.
struct Machine<'c> {
    slots: Vec<f64>,
    inputs: &'c [f64],
    tables: &'c [Vec<f64>],
    /// Each list's table, once the `const` that writes it out has run.
    lists: Vec<Option<usize>>,
    steps: u64,
    /// The members published so far, by symbol.
    published: Vec<(String, f64)>,
}

impl<'c> Machine<'c> {
    fn list(&self, k: usize, name: &str, line: usize) -> Result<&'c [f64], Diag> {
        let tables = self.tables;
        self.lists[k]
            .map(|t| tables[t].as_slice())
            .ok_or_else(|| Diag::err(line, format!("«{name}» is not a list")))
    }
    /// A row of `interp`'s table, lent: a loop calls `interp` thousands of
    /// times on the same table.
    fn row(&self, r: &Row, line: usize) -> Result<&'c [f64], Diag> {
        match r {
            Row::Table(t) => Ok(self.tables[*t].as_slice()),
            Row::List(k, name) => self.list(*k, name, line),
            Row::Not => Err(Diag::err(line, "interp needs its table")),
        }
    }
    fn pass(&mut self, line: usize) -> Result<(), Diag> {
        self.steps += 1;
        if self.steps > MAX_STEPS {
            return Err(Diag::err(line, "too many loop steps"));
        }
        Ok(())
    }
}

/// A number, worked out from the values as they stand.
type Num = Box<dyn for<'c> Fn(&mut Machine<'c>) -> Result<f64, Diag> + Send + Sync>;
/// A condition, likewise.
type Cond = Box<dyn for<'c> Fn(&mut Machine<'c>) -> Result<bool, Diag> + Send + Sync>;
/// A statement: run, it goes on or ends the method.
type Run = Box<dyn for<'c> Fn(&mut Machine<'c>) -> Result<Flow, Diag> + Send + Sync>;
/// A function of the kernel's, applied to its arguments' values.
type Apply = Box<dyn Fn(&[f64]) -> Result<f64, Diag> + Send + Sync>;

/// An operand: a leaf read where it is used, or any other number. Most
/// operands are a literal, a named value or an input, and reading one in
/// place saves a call for every one of them.
enum Opnd {
    Const(f64),
    Slot(usize),
    Input(usize),
    Num(Num),
}

#[inline]
fn fetch(o: &Opnd, m: &mut Machine<'_>) -> Result<f64, Diag> {
    match o {
        Opnd::Const(v) => Ok(*v),
        Opnd::Slot(k) => Ok(m.slots[*k]),
        Opnd::Input(k) => Ok(m.inputs[*k]),
        Opnd::Num(f) => f(m),
    }
}

fn block(body: &[Run], m: &mut Machine<'_>) -> Result<Flow, Diag> {
    for s in body {
        if let Flow::Done(o) = s(m)? {
            return Ok(Flow::Done(o));
        }
    }
    Ok(Flow::Next)
}

/// Whether an expression is a condition rather than a number.
fn is_cond(e: &Expr) -> bool {
    match e {
        Expr::Bool { .. } | Expr::Not(_) => true,
        Expr::Bin { op, .. } => matches!(op, BinOp::And | BinOp::Or) || op.is_compare(),
        _ => false,
    }
}

/// A method, read once into the form it runs in, for inputs of these names.
pub struct Compiled {
    inputs: Vec<String>,
    body: Vec<Run>,
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

    /// An expression where a number belongs, as an operand.
    fn opnd(&mut self, e: &'p Expr) -> Opnd {
        match e {
            Expr::Num { si, .. } => Opnd::Const(*si),
            Expr::Var { name, .. } => match self.place(name) {
                Place::Slot(k) => Opnd::Slot(k),
                Place::Input(k) => Opnd::Input(k),
                Place::Constant(v) => Opnd::Const(v),
                Place::Nowhere(_) => Opnd::Num(self.num(e)),
            },
            _ => Opnd::Num(self.num(e)),
        }
    }

    /// An expression where a number belongs.
    fn num(&mut self, e: &'p Expr) -> Num {
        let line = e.line();
        if is_cond(e) {
            // Worked out first, as the text has it, and only then refused.
            let c = self.cond(e);
            return Box::new(move |m| {
                c(m)?;
                Err(Diag::err(line, "a condition where a number belongs"))
            });
        }
        match e {
            Expr::Num { si, .. } => {
                let v = *si;
                Box::new(move |_| Ok(v))
            }
            Expr::Var { name, .. } => match self.place(name) {
                Place::Slot(k) => Box::new(move |m| Ok(m.slots[k])),
                Place::Input(k) => Box::new(move |m| Ok(m.inputs[k])),
                Place::Constant(v) => Box::new(move |_| Ok(v)),
                Place::Nowhere(name) => {
                    Box::new(move |_| Err(Diag::err(line, format!("«{name}» has no value"))))
                }
            },
            Expr::Neg(x) => {
                let f = self.opnd(x);
                Box::new(move |m| Ok(-fetch(&f, m)?))
            }
            Expr::Table { .. } => Box::new(move |_| {
                Err(Diag::err(
                    line,
                    "a list written out stands in a const, or inside interp(…)",
                ))
            }),
            Expr::Index { list, index, .. } => {
                let k = self.list(list);
                let name = list.clone();
                let i = self.opnd(index);
                Box::new(move |m| {
                    let i = fetch(&i, m)?;
                    rt::at(m.list(k, &name, line)?, i, line as u32).map_err(rt_diag)
                })
            }
            Expr::Bin { op, l, r, .. } => {
                let (a, b) = (self.opnd(l), self.opnd(r));
                let l32 = line as u32;
                match op {
                    BinOp::Add => Box::new(move |m| {
                        let x = fetch(&a, m)?;
                        Ok(x + fetch(&b, m)?)
                    }),
                    BinOp::Sub => Box::new(move |m| {
                        let x = fetch(&a, m)?;
                        Ok(x - fetch(&b, m)?)
                    }),
                    BinOp::Mul => Box::new(move |m| {
                        let x = fetch(&a, m)?;
                        Ok(x * fetch(&b, m)?)
                    }),
                    BinOp::Div => Box::new(move |m| {
                        let x = fetch(&a, m)?;
                        let y = fetch(&b, m)?;
                        rt::div(x, y, l32).map_err(rt_diag)
                    }),
                    BinOp::Pow => Box::new(move |m| {
                        let x = fetch(&a, m)?;
                        Ok(rt::pow(x, fetch(&b, m)?))
                    }),
                    _ => unreachable!("a comparison is a condition"),
                }
            }
            Expr::Call { name, args, .. } if name == "interp" => {
                let x = self.opnd(&args[0]);
                let (xs, ys) = (self.row(&args[1]), self.row(&args[2]));
                Box::new(move |m| {
                    let x = fetch(&x, m)?;
                    let (xs, ys) = (m.row(&xs, line)?, m.row(&ys, line)?);
                    Ok(pmath::interp(x, xs, ys))
                })
            }
            Expr::Call { name, args, .. } if name == "len" => match args.first() {
                Some(Expr::Var { name: l, .. }) => {
                    let (k, name) = (self.list(l), l.clone());
                    Box::new(move |m| Ok(m.list(k, &name, line)?.len() as f64))
                }
                _ => Box::new(move |_| Err(Diag::err(line, "len takes the name of a list"))),
            },
            Expr::Call { name, args, .. } => self.call(name, args, line),
            Expr::Bool { .. } | Expr::Not(_) => unreachable!("a condition"),
        }
    }

    /// A call: every argument worked out, in order, and then the function —
    /// the language's own first, since no kernel function shares a name with
    /// one.
    fn call(&mut self, name: &str, args: &'p [Expr], line: usize) -> Num {
        let l32 = line as u32;
        match (implementation(name), args.len()) {
            (Some(Impl::Plain1(f)), 1) => {
                let x = self.opnd(&args[0]);
                return Box::new(move |m| Ok(f(fetch(&x, m)?)));
            }
            (Some(Impl::Plain2(f)), 2) => {
                let (x, y) = (self.opnd(&args[0]), self.opnd(&args[1]));
                return Box::new(move |m| {
                    let p = fetch(&x, m)?;
                    Ok(f(p, fetch(&y, m)?))
                });
            }
            (Some(Impl::Checked1(f)), 1) => {
                let x = self.opnd(&args[0]);
                return Box::new(move |m| f(fetch(&x, m)?, l32).map_err(rt_diag));
            }
            (Some(Impl::Checked2(f)), 2) => {
                let (x, y) = (self.opnd(&args[0]), self.opnd(&args[1]));
                return Box::new(move |m| {
                    let p = fetch(&x, m)?;
                    let q = fetch(&y, m)?;
                    f(p, q, l32).map_err(rt_diag)
                });
            }
            _ => {}
        }
        let a: Vec<Opnd> = args.iter().map(|e| self.opnd(e)).collect();
        // Any other call: the arguments on the stack when there are few, as
        // there almost always are.
        let apply: Apply = match implementation(name) {
            Some(Impl::Plain1(f)) => Box::new(move |v| Ok(f(v[0]))),
            Some(Impl::Plain2(f)) => Box::new(move |v| Ok(f(v[0], v[1]))),
            Some(Impl::Checked1(f)) => Box::new(move |v| f(v[0], l32).map_err(rt_diag)),
            Some(Impl::Checked2(f)) => Box::new(move |v| f(v[0], v[1], l32).map_err(rt_diag)),
            None => match kernel_function(name) {
                Some(k) => Box::new(move |v| {
                    let r = (k.eval)(v);
                    if r.is_nan() && !k.refuses.is_empty() {
                        return Err(Diag::err(line, k.refuses));
                    }
                    Ok(r)
                }),
                None => {
                    let name = name.to_string();
                    Box::new(move |_| Err(Diag::err(line, format!("«{name}» is not a function"))))
                }
            },
        };
        Box::new(move |m| {
            let mut few = [0.0f64; 8];
            let mut many = Vec::new();
            let v: &[f64] = if a.len() <= few.len() {
                for (slot, e) in few.iter_mut().zip(&a) {
                    *slot = fetch(e, m)?;
                }
                &few[..a.len()]
            } else {
                for e in &a {
                    many.push(fetch(e, m)?);
                }
                &many
            };
            apply(v)
        })
    }

    /// An expression where a condition belongs.
    fn cond(&mut self, e: &'p Expr) -> Cond {
        let line = e.line();
        if !is_cond(e) {
            // Worked out first, as the text has it, and only then refused.
            let f = self.num(e);
            return Box::new(move |m| {
                f(m)?;
                Err(Diag::err(line, "a number where a condition belongs"))
            });
        }
        match e {
            Expr::Bool { value, .. } => {
                let v = *value;
                Box::new(move |_| Ok(v))
            }
            Expr::Not(x) => {
                let c = self.cond(x);
                Box::new(move |m| Ok(!c(m)?))
            }
            Expr::Bin { op, l, r, .. } => match op {
                BinOp::And => {
                    let (a, b) = (self.cond(l), self.cond(r));
                    Box::new(move |m| Ok(a(m)? && b(m)?))
                }
                BinOp::Or => {
                    let (a, b) = (self.cond(l), self.cond(r));
                    Box::new(move |m| Ok(a(m)? || b(m)?))
                }
                op => {
                    let (a, b) = (self.opnd(l), self.opnd(r));
                    let cmp: fn(f64, f64) -> bool = match op {
                        BinOp::Lt => |x, y| x < y,
                        BinOp::Le => |x, y| x <= y,
                        BinOp::Gt => |x, y| x > y,
                        BinOp::Ge => |x, y| x >= y,
                        BinOp::Eq => |x, y| x == y,
                        BinOp::Ne => |x, y| x != y,
                        _ => unreachable!("arithmetic is a number"),
                    };
                    Box::new(move |m| {
                        let x = fetch(&a, m)?;
                        let y = fetch(&b, m)?;
                        Ok(cmp(x, y))
                    })
                }
            },
            _ => unreachable!("only these are conditions"),
        }
    }

    fn block(&mut self, body: &'p [Stmt]) -> Vec<Run> {
        body.iter().map(|s| self.stmt(s)).collect()
    }
    /// A block in a scope of its own.
    fn scoped(&mut self, body: &'p [Stmt]) -> Vec<Run> {
        self.scopes.push(Vec::new());
        let b = self.block(body);
        self.scopes.pop();
        b
    }
    fn stmt(&mut self, s: &'p Stmt) -> Run {
        match s {
            Stmt::Const {
                name,
                expr: Expr::Table { si, .. },
                ..
            } => {
                let t = self.table(si);
                let k = self.list(name);
                Box::new(move |m| {
                    m.lists[k] = Some(t);
                    Ok(Flow::Next)
                })
            }
            Stmt::Let {
                name, expr, line, ..
            }
            | Stmt::Const { name, expr, line } => {
                // The value is read where the text stands, before the name
                // it is given is in scope.
                let e = self.opnd(expr);
                let (k, l32) = (self.name(name), *line as u32);
                Box::new(move |m| {
                    m.slots[k] = rt::fin(fetch(&e, m)?, l32).map_err(rt_diag)?;
                    Ok(Flow::Next)
                })
            }
            Stmt::Set { name, expr, line } => {
                let e = self.opnd(expr);
                let (k, l32) = (self.settable(name), *line as u32);
                Box::new(move |m| {
                    let v = rt::fin(fetch(&e, m)?, l32).map_err(rt_diag)?;
                    if let Some(k) = k {
                        m.slots[k] = v;
                    }
                    Ok(Flow::Next)
                })
            }
            Stmt::If {
                arms, otherwise, ..
            } => {
                let arms: Vec<(Cond, Vec<Run>)> = arms
                    .iter()
                    .map(|(c, body)| {
                        let c = self.cond(c);
                        (c, self.scoped(body))
                    })
                    .collect();
                let otherwise = otherwise.as_ref().map(|b| self.scoped(b));
                Box::new(move |m| {
                    for (cond, body) in &arms {
                        if cond(m)? {
                            return block(body, m);
                        }
                    }
                    match &otherwise {
                        Some(body) => block(body, m),
                        None => Ok(Flow::Next),
                    }
                })
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
                let (first, last, line) = (*first, *last, *line);
                Box::new(move |m| {
                    for i in first..=last {
                        m.pass(line)?;
                        m.slots[slot] = i as f64;
                        if let Flow::Done(o) = block(&body, m)? {
                            return Ok(Flow::Done(o));
                        }
                    }
                    Ok(Flow::Next)
                })
            }
            Stmt::Each {
                var,
                list,
                body,
                line,
            } => {
                let k = self.list(list);
                let name = list.clone();
                self.scopes.push(Vec::new());
                let slot = self.name(var);
                let body = self.block(body);
                self.scopes.pop();
                let line = *line;
                Box::new(move |m| {
                    let entries = m.list(k, &name, line)?;
                    for &v in entries {
                        m.pass(line)?;
                        m.slots[slot] = v;
                        if let Flow::Done(o) = block(&body, m)? {
                            return Ok(Flow::Done(o));
                        }
                    }
                    Ok(Flow::Next)
                })
            }
            Stmt::While {
                cond,
                max,
                body,
                line,
            } => {
                let cond = self.cond(cond);
                let body = self.scoped(body);
                let (max, line) = (*max, *line);
                Box::new(move |m| {
                    let mut passes = 0i64;
                    while cond(m)? {
                        if passes == max {
                            return Ok(Flow::Done(Outcome::Refused {
                                line,
                                reason: unsettled(line, max),
                            }));
                        }
                        passes += 1;
                        m.pass(line)?;
                        if let Flow::Done(o) = block(&body, m)? {
                            return Ok(Flow::Done(o));
                        }
                    }
                    Ok(Flow::Next)
                })
            }
            Stmt::Refuse { reason, line } => {
                let (reason, line) = (reason.clone(), *line);
                Box::new(move |_| {
                    Ok(Flow::Done(Outcome::Refused {
                        line,
                        reason: reason.clone(),
                    }))
                })
            }
            Stmt::Return { expr, line } => {
                let (e, l32) = (self.opnd(expr), *line as u32);
                Box::new(move |m| {
                    let v = rt::fin(fetch(&e, m)?, l32).map_err(rt_diag)?;
                    Ok(Flow::Done(Outcome::Answer(v)))
                })
            }
            Stmt::Publish { name, expr, line } => {
                let (name, e, l32) = (name.clone(), self.num(expr), *line as u32);
                Box::new(move |m| {
                    let v = rt::fin(e(m)?, l32).map_err(rt_diag)?;
                    m.published.push((name.clone(), v));
                    Ok(Flow::Next)
                })
            }
        }
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
        // first line, never carried into the arithmetic to fault there.
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
            slots: vec![0.0; self.slots],
            inputs,
            tables: &self.tables,
            lists: vec![None; self.lists],
            steps: 0,
            published: Vec::new(),
        };
        match block(&self.body, &mut m)? {
            Flow::Done(o @ Outcome::Answer(_)) => Ok((o, m.published)),
            Flow::Done(o) => Ok((o, Vec::new())),
            Flow::Next => Err(Diag::err(0, "the method ended without return or refuse")),
        }
    }
}

/// Why a loop that did not settle refuses.
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
