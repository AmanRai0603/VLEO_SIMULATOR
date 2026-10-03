//! The method language: the translation into Rust.

use super::*;

/// Rust's reserved words. A method name that is one is written as a raw
/// identifier; the four that cannot be are refused by the checker.
const RUST_WORDS: &[&str] = &[
    "as", "async", "await", "break", "continue", "dyn", "enum", "extern", "fn", "impl", "in",
    "loop", "match", "mod", "move", "mut", "pub", "ref", "static", "struct", "trait", "type",
    "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "gen", "macro",
    "override", "priv", "try", "typeof", "unsized", "virtual", "yield",
];
pub(super) const RUST_UNRAW: &[&str] = &["self", "Self", "super", "crate"];

fn ident(name: &str) -> String {
    if RUST_WORDS.contains(&name) {
        format!("r#{name}")
    } else {
        name.to_string()
    }
}

fn lit(v: f64) -> String {
    let t = format!("{v:?}");
    if v < 0.0 {
        format!("({t})")
    } else {
        t
    }
}

fn rstr(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

struct Rust {
    mutable: BTreeSet<String>,
    depth: usize,
    /// The node's published members, by symbol, in declared order: each
    /// `publish` writes its slot, and `return` hands the slots back beside the
    /// answer. Empty for a node with one answer.
    publishes: Vec<String>,
}

impl Rust {
    fn expr(&self, e: &Expr) -> String {
        match e {
            Expr::Num { si, .. } => lit(*si),
            Expr::Bool { value, .. } => value.to_string(),
            Expr::Var { name, .. } => match KERNEL_CONSTANTS.iter().find(|c| c.name == name) {
                Some(c) => c.rust.to_string(),
                None => ident(name),
            },
            Expr::Neg(x) => format!("(-{})", self.expr(x)),
            Expr::Not(x) => format!("(!{})", self.expr(x)),
            Expr::Table { si, .. } => format!(
                "&[{}]",
                si.iter().map(|v| lit(*v)).collect::<Vec<_>>().join(", ")
            ),
            Expr::Bin { op, l, r, line } => {
                let (a, b) = (self.expr(l), self.expr(r));
                match op {
                    BinOp::Div => format!("rt::div({a}, {b}, {line})?"),
                    BinOp::Pow => format!("rt::pow({a}, {b})"),
                    BinOp::And => format!("({a} && {b})"),
                    BinOp::Or => format!("({a} || {b})"),
                    _ => format!("({a} {} {b})", op.symbol()),
                }
            }
            Expr::Call { name, args, line } => {
                let a: Vec<String> = args.iter().map(|x| self.expr(x)).collect();
                if name == "interp" {
                    return format!("pmath::interp({}, {}, {})", a[0], a[1], a[2]);
                }
                if let Some(k) = kernel_function(name) {
                    return format!("({})", translate(k, &a));
                }
                match implementation(name) {
                    Some(Impl::Plain1(_, p)) => format!("{p}({})", a[0]),
                    Some(Impl::Plain2(_, p)) => format!("{p}({}, {})", a[0], a[1]),
                    Some(Impl::Checked1(_, p)) => format!("{p}({}, {line})?", a[0]),
                    Some(Impl::Checked2(_, p)) => format!("{p}({}, {}, {line})?", a[0], a[1]),
                    None => format!("/* no function {name} */ f64::NAN"),
                }
            }
        }
    }

    fn block(&mut self, body: &[Stmt], o: &mut String) {
        for s in body {
            self.stmt(s, o);
        }
    }

    fn stmt(&mut self, s: &Stmt, o: &mut String) {
        match s {
            Stmt::Let {
                name, expr, line, ..
            }
            | Stmt::Const { name, expr, line } => {
                let m = if self.mutable.contains(name) {
                    "mut "
                } else {
                    ""
                };
                let _ = writeln!(
                    o,
                    "let {m}{}: f64 = rt::fin({}, {line})?;",
                    ident(name),
                    self.expr(expr)
                );
            }
            Stmt::Set { name, expr, line } => {
                let _ = writeln!(
                    o,
                    "{} = rt::fin({}, {line})?;",
                    ident(name),
                    self.expr(expr)
                );
            }
            Stmt::If {
                arms, otherwise, ..
            } => {
                for (i, (c, body)) in arms.iter().enumerate() {
                    let _ = writeln!(
                        o,
                        "{}if {} {{",
                        if i == 0 { "" } else { "} else " },
                        self.expr(c)
                    );
                    self.block(body, o);
                }
                if let Some(body) = otherwise {
                    o.push_str("} else {\n");
                    self.block(body, o);
                }
                o.push_str("}\n");
            }
            Stmt::For {
                var,
                first,
                last,
                body,
                ..
            } => {
                self.depth += 1;
                let i = format!("step_{}", self.depth);
                let _ = writeln!(o, "for {i} in ({first}_i64)..=({last}_i64) {{");
                let _ = writeln!(o, "let {}: f64 = {i} as f64;", ident(var));
                self.block(body, o);
                o.push_str("}\n");
                self.depth -= 1;
            }
            Stmt::While {
                cond,
                max,
                body,
                line,
            } => {
                self.depth += 1;
                let n = format!("passes_{}", self.depth);
                let _ = writeln!(o, "let mut {n}: i64 = 0;");
                let _ = writeln!(o, "while {} {{", self.expr(cond));
                let _ = writeln!(
                    o,
                    "if {n} == {max}_i64 {{ return Err(MethodError::Refused({})); }}",
                    rstr(&super::run::unsettled(*line, *max))
                );
                let _ = writeln!(o, "{n} += 1;");
                self.block(body, o);
                o.push_str("}\n");
                self.depth -= 1;
            }
            Stmt::Refuse { reason, .. } => {
                let _ = writeln!(o, "return Err(MethodError::Refused({}));", rstr(reason));
            }
            Stmt::Return { expr, line } => {
                if self.publishes.is_empty() {
                    let _ = writeln!(o, "return Ok(rt::fin({}, {line})?);", self.expr(expr));
                } else {
                    let _ = writeln!(
                        o,
                        "return Ok((rt::fin({}, {line})?, published));",
                        self.expr(expr)
                    );
                }
            }
            Stmt::Publish { name, expr, line } => {
                let slot = self.publishes.iter().position(|n| n == name).unwrap_or(0);
                let _ = writeln!(
                    o,
                    "published[{slot}] = rt::fin({}, {line})?; // {name}",
                    self.expr(expr)
                );
            }
        }
    }
}

fn set_targets(body: &[Stmt], out: &mut BTreeSet<String>) {
    for s in body {
        match s {
            Stmt::Set { name, .. } => {
                out.insert(name.clone());
            }
            Stmt::If {
                arms, otherwise, ..
            } => {
                for (_, b) in arms {
                    set_targets(b, out);
                }
                if let Some(b) = otherwise {
                    set_targets(b, out);
                }
            }
            Stmt::For { body, .. } | Stmt::While { body, .. } => set_targets(body, out),
            _ => {}
        }
    }
}

/// The method as a kernel function: `vleo_core::physics::methods::<node>`.
///
/// BY RULE, NOT BY JUDGEMENT. Each construct has exactly one translation, the
/// checked operations are the same `vleo_units::method_rt` functions the
/// interpreter calls, and a value is checked for finiteness at the same places
/// — so the function gives the interpreter's answer to the last bit, which is
/// what the node's generated translation test asserts. `inputs` is the
/// parameter order, the node's declared input order; every value is SI.
pub fn to_rust(p: &Program, node: &str, source: &str, src: &str, inputs: &[String]) -> String {
    to_rust_publishing(p, node, source, src, inputs, &[])
}

/// The same, for a node that publishes members beside its answer: `evaluate`
/// returns the answer and an array of the members, in `publishes` order (the
/// order the sheet declares them), each in SI. The checker has already held
/// that every member is published, once, before any return.
pub fn to_rust_publishing(
    p: &Program,
    node: &str,
    source: &str,
    src: &str,
    inputs: &[String],
    publishes: &[String],
) -> String {
    let mut mutable = BTreeSet::new();
    set_targets(&p.body, &mut mutable);
    let mut r = Rust {
        mutable,
        depth: 0,
        publishes: publishes.to_vec(),
    };
    let mut o = String::new();
    let _ = writeln!(
        o,
        "//! GENERATED from the method of `{node}` by `cargo xtask docs`, translated by\n\
         //! the fixed rules in crates/vleo-sheet/src/method.rs. Do not edit: the method\n\
         //! is changed on the node's form, and this is written again from it.\n"
    );
    o.push_str(
        "#![allow(clippy::all, clippy::float_cmp, clippy::cast_precision_loss, unreachable_code, \
         unused_imports, unused_mut, unused_variables, unused_parens, non_snake_case)]\n\n",
    );
    o.push_str("use vleo_units::constants::*;\nuse vleo_units::method_rt::{self as rt, MethodError};\nuse vleo_units::pmath;\n\n");
    let _ = writeln!(
        o,
        "/// The method of `{node}`, source `{source}`:\n///\n/// ```text"
    );
    for l in src.lines() {
        let _ = writeln!(o, "/// {l}");
    }
    o.push_str("/// ```\n");
    let params: Vec<String> = inputs
        .iter()
        .map(|n| format!("{}: f64", ident(n)))
        .collect();
    if publishes.is_empty() {
        let _ = writeln!(
            o,
            "pub fn evaluate({}) -> Result<f64, MethodError> {{",
            params.join(", ")
        );
    } else {
        let _ = writeln!(
            o,
            "/// Returns the answer, and the published members in this order: {}.\n\
             pub fn evaluate({}) -> Result<(f64, [f64; {}]), MethodError> {{\n\
             let mut published = [0.0_f64; {}];",
            publishes.join(", "),
            params.join(", "),
            publishes.len(),
            publishes.len()
        );
    }
    r.block(&p.body, &mut o);
    o.push_str(
        "Err(MethodError::Degenerate { line: 0, what: \"the method ended without an answer\" })\n}\n",
    );
    o
}
