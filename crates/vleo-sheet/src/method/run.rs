//! The method language: running: the interpreter.

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

struct Machine<'a> {
    scopes: Vec<BTreeMap<String, f64>>,
    inputs: &'a [(String, f64)],
    steps: u64,
}

impl Machine<'_> {
    fn get(&self, name: &str, line: usize) -> Result<f64, Diag> {
        for s in self.scopes.iter().rev() {
            if let Some(v) = s.get(name) {
                return Ok(*v);
            }
        }
        if let Some((_, v)) = self.inputs.iter().find(|(n, _)| n == name) {
            return Ok(*v);
        }
        if let Some((v, _)) = kernel_constant(name) {
            return Ok(v);
        }
        Err(Diag::err(line, format!("«{name}» has no value")))
    }
    fn set(&mut self, name: &str, v: f64) {
        for s in self.scopes.iter_mut().rev() {
            if let Some(slot) = s.get_mut(name) {
                *slot = v;
                return;
            }
        }
    }
    fn num(&mut self, e: &Expr) -> Result<f64, Diag> {
        match self.eval(e)? {
            Val::Num(v) => Ok(v),
            Val::Bool(_) => Err(Diag::err(e.line(), "a condition where a number belongs")),
        }
    }
    fn cond(&mut self, e: &Expr) -> Result<bool, Diag> {
        match self.eval(e)? {
            Val::Bool(b) => Ok(b),
            Val::Num(_) => Err(Diag::err(e.line(), "a number where a condition belongs")),
        }
    }
    fn eval(&mut self, e: &Expr) -> Result<Val, Diag> {
        let v = match e {
            Expr::Num { si, .. } => *si,
            Expr::Bool { value, .. } => return Ok(Val::Bool(*value)),
            Expr::Var { name, line } => self.get(name, *line)?,
            Expr::Neg(x) => -self.num(x)?,
            Expr::Not(x) => return Ok(Val::Bool(!self.cond(x)?)),
            Expr::Table { line, .. } => {
                return Err(Diag::err(*line, "a table stands only inside interp(…)"))
            }
            Expr::Bin { op, l, r, line } => {
                let line = *line;
                match op {
                    BinOp::And => {
                        return Ok(Val::Bool(self.cond(l)? && self.cond(r)?));
                    }
                    BinOp::Or => {
                        return Ok(Val::Bool(self.cond(l)? || self.cond(r)?));
                    }
                    _ => {}
                }
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
                    BinOp::And | BinOp::Or => unreachable!("handled above"),
                }
            }
            Expr::Call { name, args, line } => self.call(name, args, *line)?,
        };
        // Finiteness is checked where a value is named or returned — see
        // `rt::fin` — exactly as the translated code checks it.
        Ok(Val::Num(v))
    }
    fn call(&mut self, name: &str, args: &[Expr], line: usize) -> Result<f64, Diag> {
        if name == "interp" {
            let x = self.num(&args[0])?;
            let (Expr::Table { si: xs, .. }, Expr::Table { si: ys, .. }) = (&args[1], &args[2])
            else {
                return Err(Diag::err(line, "interp needs its table written out"));
            };
            return Ok(pmath::interp(x, xs, ys));
        }
        let mut a = Vec::with_capacity(args.len());
        for e in args {
            a.push(self.num(e)?);
        }
        let l = line as u32;
        match implementation(name) {
            Some(Impl::Plain1(f, _)) => Ok(f(a[0])),
            Some(Impl::Plain2(f, _)) => Ok(f(a[0], a[1])),
            Some(Impl::Checked1(f, _)) => f(a[0], l).map_err(rt_diag),
            Some(Impl::Checked2(f, _)) => f(a[0], a[1], l).map_err(rt_diag),
            None => Err(Diag::err(line, format!("«{name}» is not a function"))),
        }
    }
    fn block(&mut self, body: &[Stmt]) -> Result<Flow, Diag> {
        for s in body {
            if let Flow::Done(o) = self.stmt(s)? {
                return Ok(Flow::Done(o));
            }
        }
        Ok(Flow::Next)
    }
    fn stmt(&mut self, s: &Stmt) -> Result<Flow, Diag> {
        match s {
            Stmt::Let {
                name, expr, line, ..
            }
            | Stmt::Const { name, expr, line } => {
                let v = rt::fin(self.num(expr)?, *line as u32).map_err(rt_diag)?;
                self.scopes
                    .last_mut()
                    .expect("a scope")
                    .insert(name.clone(), v);
            }
            Stmt::Set { name, expr, line } => {
                let v = rt::fin(self.num(expr)?, *line as u32).map_err(rt_diag)?;
                self.set(name, v);
            }
            Stmt::If {
                arms, otherwise, ..
            } => {
                for (cond, body) in arms {
                    if self.cond(cond)? {
                        self.scopes.push(BTreeMap::new());
                        let f = self.block(body);
                        self.scopes.pop();
                        return f;
                    }
                }
                if let Some(body) = otherwise {
                    self.scopes.push(BTreeMap::new());
                    let f = self.block(body);
                    self.scopes.pop();
                    return f;
                }
            }
            Stmt::For {
                var,
                first,
                last,
                body,
                line,
            } => {
                for i in *first..=*last {
                    self.steps += 1;
                    if self.steps > MAX_STEPS {
                        return Err(Diag::err(*line, "too many loop steps"));
                    }
                    let mut scope = BTreeMap::new();
                    scope.insert(var.clone(), i as f64);
                    self.scopes.push(scope);
                    let f = self.block(body);
                    self.scopes.pop();
                    if let Flow::Done(o) = f? {
                        return Ok(Flow::Done(o));
                    }
                }
            }
            Stmt::Refuse { reason, line } => {
                return Ok(Flow::Done(Outcome::Refused {
                    line: *line,
                    reason: reason.clone(),
                }))
            }
            Stmt::Return { expr, line } => {
                let v = rt::fin(self.num(expr)?, *line as u32).map_err(rt_diag)?;
                return Ok(Flow::Done(Outcome::Answer(v)));
            }
        }
        Ok(Flow::Next)
    }
}

/// Run a checked method on one set of inputs, each in the SI unit of its
/// quantity. An error is a fault in the method or its inputs — a square root
/// of a negative number, say — and never a refusal: only the method's own
/// `refuse` refuses.
pub fn run(p: &Program, inputs: &[(String, f64)]) -> Result<Outcome, Diag> {
    let mut m = Machine {
        scopes: vec![BTreeMap::new()],
        inputs,
        steps: 0,
    };
    match m.block(&p.body)? {
        Flow::Done(o) => Ok(o),
        Flow::Next => Err(Diag::err(0, "the method ended without return or refuse")),
    }
}
