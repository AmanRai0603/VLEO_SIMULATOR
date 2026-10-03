//! The method language: reading the text: the lexer and the parser.

use super::*;

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Ident(String),
    Str(String),
    Unit(String),
    Sym(&'static str),
    Newline,
}

#[derive(Clone, Debug)]
struct Token {
    tok: Tok,
    line: usize,
}

const SYMBOLS: &[&str] = &[
    "<=", ">=", "==", "!=", "+", "-", "*", "/", "^", "(", ")", "[", "]", ",", "=", "<", ">", ":",
];

fn lex(src: &str) -> Result<Vec<Token>, Diag> {
    let mut out: Vec<Token> = Vec::new();
    let mut depth = 0i32;
    for (i, raw) in src.lines().enumerate() {
        let line = i + 1;
        let text = match raw.find('#') {
            Some(p) if !raw[..p].contains('"') || raw[..p].matches('"').count() % 2 == 0 => {
                &raw[..p]
            }
            _ => raw,
        };
        let b = text.as_bytes();
        let mut p = 0usize;
        while p < b.len() {
            let c = b[p] as char;
            if c.is_whitespace() {
                p += 1;
                continue;
            }
            if c == '"' {
                let rest = &text[p + 1..];
                let Some(q) = rest.find('"') else {
                    return Err(Diag::err(line, "a quotation is not closed on its line"));
                };
                out.push(Token {
                    tok: Tok::Str(rest[..q].to_string()),
                    line,
                });
                p += q + 2;
                continue;
            }
            if c.is_ascii_digit() || (c == '.' && p + 1 < b.len() && b[p + 1].is_ascii_digit()) {
                let start = p;
                while p < b.len() && ((b[p] as char).is_ascii_digit() || b[p] == b'.') {
                    p += 1;
                }
                if p < b.len() && (b[p] == b'e' || b[p] == b'E') {
                    let save = p;
                    p += 1;
                    if p < b.len() && (b[p] == b'+' || b[p] == b'-') {
                        p += 1;
                    }
                    if p < b.len() && (b[p] as char).is_ascii_digit() {
                        while p < b.len() && (b[p] as char).is_ascii_digit() {
                            p += 1;
                        }
                    } else {
                        p = save;
                    }
                }
                let s = &text[start..p];
                let v: f64 = s
                    .parse()
                    .map_err(|_| Diag::err(line, format!("«{s}» is not a number")))?;
                out.push(Token {
                    tok: Tok::Num(v),
                    line,
                });
                continue;
            }
            if c.is_ascii_alphabetic() || c == '_' {
                let start = p;
                while p < b.len() && ((b[p] as char).is_ascii_alphanumeric() || b[p] == b'_') {
                    p += 1;
                }
                out.push(Token {
                    tok: Tok::Ident(text[start..p].to_string()),
                    line,
                });
                continue;
            }
            if c == '[' {
                // A bracket straight after a number, or after a table's closing
                // bracket, is a unit; anywhere else it opens a table.
                let after_value = matches!(
                    out.last(),
                    Some(Token { tok: Tok::Num(_), line: l }) | Some(Token { tok: Tok::Sym("]"), line: l })
                        if *l == line
                );
                if after_value {
                    let rest = &text[p + 1..];
                    let Some(q) = rest.find(']') else {
                        return Err(Diag::err(line, "a unit's [ is not closed on its line"));
                    };
                    out.push(Token {
                        tok: Tok::Unit(rest[..q].trim().to_string()),
                        line,
                    });
                    p += q + 2;
                    continue;
                }
            }
            let Some(sym) = SYMBOLS.iter().find(|s| text[p..].starts_with(**s)) else {
                return Err(Diag::err(
                    line,
                    format!("«{c}» is not part of the method language"),
                ));
            };
            match *sym {
                "(" | "[" => depth += 1,
                ")" | "]" => depth -= 1,
                _ => {}
            }
            out.push(Token {
                tok: Tok::Sym(sym),
                line,
            });
            p += sym.len();
        }
        if depth <= 0 {
            depth = 0;
            if !matches!(
                out.last(),
                None | Some(Token {
                    tok: Tok::Newline,
                    ..
                })
            ) {
                out.push(Token {
                    tok: Tok::Newline,
                    line,
                });
            }
        }
    }
    if !matches!(
        out.last(),
        None | Some(Token {
            tok: Tok::Newline,
            ..
        })
    ) {
        let line = out.last().map(|t| t.line).unwrap_or(0);
        out.push(Token {
            tok: Tok::Newline,
            line,
        });
    }
    Ok(out)
}

struct Parser {
    toks: Vec<Token>,
    pos: usize,
}

type PResult<T> = Result<T, Diag>;

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos).map(|t| &t.tok)
    }
    fn line(&self) -> usize {
        self.toks
            .get(self.pos)
            .or_else(|| self.toks.last())
            .map(|t| t.line)
            .unwrap_or(0)
    }
    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).map(|t| t.tok.clone());
        self.pos += 1;
        t
    }
    fn is_sym(&self, s: &str) -> bool {
        matches!(self.peek(), Some(Tok::Sym(x)) if *x == s)
    }
    fn is_kw(&self, k: &str) -> bool {
        matches!(self.peek(), Some(Tok::Ident(x)) if x == k)
    }
    fn expect_sym(&mut self, s: &str) -> PResult<()> {
        if self.is_sym(s) {
            self.pos += 1;
            Ok(())
        } else {
            Err(Diag::err(self.line(), format!("expected «{s}» here")))
        }
    }
    fn expect_kw(&mut self, k: &str) -> PResult<()> {
        if self.is_kw(k) {
            self.pos += 1;
            Ok(())
        } else {
            Err(Diag::err(self.line(), format!("expected «{k}» here")))
        }
    }
    fn end_of_line(&mut self) -> PResult<()> {
        match self.peek() {
            Some(Tok::Newline) => {
                self.pos += 1;
                Ok(())
            }
            None => Ok(()),
            _ => Err(Diag::err(
                self.line(),
                "one statement per line — something follows the end of this one",
            )),
        }
    }
    fn name(&mut self, what: &str) -> PResult<String> {
        let line = self.line();
        match self.next() {
            Some(Tok::Ident(n)) if !KEYWORDS.contains(&n.as_str()) => Ok(n),
            Some(Tok::Ident(n)) => Err(Diag::err(
                line,
                format!("«{n}» is a word of the language and cannot be {what}"),
            )),
            _ => Err(Diag::err(line, format!("expected {what} here"))),
        }
    }

    fn block(&mut self, stops: &[&str]) -> PResult<Vec<Stmt>> {
        let mut body = Vec::new();
        loop {
            while matches!(self.peek(), Some(Tok::Newline)) {
                self.pos += 1;
            }
            match self.peek() {
                None => {
                    if stops.is_empty() {
                        return Ok(body);
                    }
                    return Err(Diag::err(
                        self.line(),
                        "a block is not closed — every if, for and while needs its «end»",
                    ));
                }
                Some(Tok::Ident(k)) if stops.contains(&k.as_str()) => return Ok(body),
                _ => body.push(self.stmt()?),
            }
        }
    }

    fn stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        let Some(Tok::Ident(word)) = self.peek().cloned() else {
            return Err(Diag::err(
                line,
                "a statement starts with let, const, set, if, for, while, refuse or return",
            ));
        };
        match word.as_str() {
            "let" => {
                self.pos += 1;
                let name = self.name("a name")?;
                let quantity = if self.is_sym(":") {
                    self.pos += 1;
                    Some(self.name("a quantity type")?)
                } else {
                    None
                };
                self.expect_sym("=")?;
                let expr = self.expr()?;
                self.end_of_line()?;
                Ok(Stmt::Let {
                    name,
                    quantity,
                    expr,
                    line,
                })
            }
            "const" => {
                self.pos += 1;
                let name = self.name("a name")?;
                self.expect_sym("=")?;
                let expr = self.expr()?;
                if expr.literal().is_none() {
                    return Err(Diag::err(
                        line,
                        "a const is one number with its unit — use let for anything computed",
                    ));
                }
                self.end_of_line()?;
                Ok(Stmt::Const { name, expr, line })
            }
            "set" => {
                self.pos += 1;
                let name = self.name("a name")?;
                self.expect_sym("=")?;
                let expr = self.expr()?;
                self.end_of_line()?;
                Ok(Stmt::Set { name, expr, line })
            }
            "if" => {
                self.pos += 1;
                let mut arms = Vec::new();
                let cond = self.expr()?;
                self.expect_kw("then")?;
                self.end_of_line()?;
                let body = self.block(&["else", "end"])?;
                arms.push((cond, body));
                let mut otherwise = None;
                loop {
                    if self.is_kw("end") {
                        self.pos += 1;
                        self.end_of_line()?;
                        break;
                    }
                    self.expect_kw("else")?;
                    if self.is_kw("if") {
                        self.pos += 1;
                        let cond = self.expr()?;
                        self.expect_kw("then")?;
                        self.end_of_line()?;
                        let body = self.block(&["else", "end"])?;
                        arms.push((cond, body));
                    } else {
                        self.end_of_line()?;
                        otherwise = Some(self.block(&["end"])?);
                        self.expect_kw("end")?;
                        self.end_of_line()?;
                        break;
                    }
                }
                Ok(Stmt::If {
                    arms,
                    otherwise,
                    line,
                })
            }
            "for" => {
                self.pos += 1;
                let var = self.name("a loop name")?;
                self.expect_sym("=")?;
                let first = self.whole()?;
                self.expect_kw("to")?;
                let last = self.whole()?;
                self.end_of_line()?;
                let body = self.block(&["end"])?;
                self.expect_kw("end")?;
                self.end_of_line()?;
                Ok(Stmt::For {
                    var,
                    first,
                    last,
                    body,
                    line,
                })
            }
            "while" => {
                self.pos += 1;
                let cond = self.expr()?;
                for w in ["at", "most"] {
                    if !self.is_kw(w) {
                        return Err(Diag::err(
                            line,
                            "a while loop says how often it may run: while CONDITION at most N times",
                        ));
                    }
                    self.pos += 1;
                }
                let max = self.whole()?;
                self.expect_kw("times")?;
                self.end_of_line()?;
                let body = self.block(&["end"])?;
                self.expect_kw("end")?;
                self.end_of_line()?;
                Ok(Stmt::While {
                    cond,
                    max,
                    body,
                    line,
                })
            }
            "refuse" => {
                self.pos += 1;
                let reason = match self.next() {
                    Some(Tok::Str(s)) => s,
                    _ => {
                        return Err(Diag::err(
                            line,
                            "refuse is followed by its reason in quotation marks",
                        ))
                    }
                };
                self.end_of_line()?;
                Ok(Stmt::Refuse { reason, line })
            }
            "return" => {
                self.pos += 1;
                let expr = self.expr()?;
                self.end_of_line()?;
                Ok(Stmt::Return { expr, line })
            }
            "else" | "end" => Err(Diag::err(
                line,
                format!("«{word}» with no if, for or while open"),
            )),
            _ => Err(Diag::err(
                line,
                format!(
                    "a statement starts with let, const, set, if, for, while, refuse or return — not «{word}»"
                ),
            )),
        }
    }

    fn whole(&mut self) -> PResult<i64> {
        let line = self.line();
        let neg = if self.is_sym("-") {
            self.pos += 1;
            true
        } else {
            false
        };
        match self.next() {
            Some(Tok::Num(v)) if v.fract() == 0.0 && v.abs() < 1e9 => {
                Ok(if neg { -(v as i64) } else { v as i64 })
            }
            _ => Err(Diag::err(line, "a loop runs between two whole numbers")),
        }
    }

    fn expr(&mut self) -> PResult<Expr> {
        let mut l = self.and_expr()?;
        while self.is_kw("or") {
            let line = self.line();
            self.pos += 1;
            let r = self.and_expr()?;
            l = Expr::Bin {
                op: BinOp::Or,
                l: Box::new(l),
                r: Box::new(r),
                line,
            };
        }
        Ok(l)
    }
    fn and_expr(&mut self) -> PResult<Expr> {
        let mut l = self.not_expr()?;
        while self.is_kw("and") {
            let line = self.line();
            self.pos += 1;
            let r = self.not_expr()?;
            l = Expr::Bin {
                op: BinOp::And,
                l: Box::new(l),
                r: Box::new(r),
                line,
            };
        }
        Ok(l)
    }
    fn not_expr(&mut self) -> PResult<Expr> {
        if self.is_kw("not") {
            self.pos += 1;
            return Ok(Expr::Not(Box::new(self.not_expr()?)));
        }
        self.compare()
    }
    fn compare(&mut self) -> PResult<Expr> {
        let l = self.sum()?;
        let op = match self.peek() {
            Some(Tok::Sym("<")) => BinOp::Lt,
            Some(Tok::Sym("<=")) => BinOp::Le,
            Some(Tok::Sym(">")) => BinOp::Gt,
            Some(Tok::Sym(">=")) => BinOp::Ge,
            Some(Tok::Sym("==")) => BinOp::Eq,
            Some(Tok::Sym("!=")) => BinOp::Ne,
            _ => return Ok(l),
        };
        let line = self.line();
        self.pos += 1;
        let r = self.sum()?;
        if matches!(
            self.peek(),
            Some(Tok::Sym("<" | "<=" | ">" | ">=" | "==" | "!="))
        ) {
            return Err(Diag::err(
                line,
                "comparisons do not chain — write a < b and b < c",
            ));
        }
        Ok(Expr::Bin {
            op,
            l: Box::new(l),
            r: Box::new(r),
            line,
        })
    }
    fn sum(&mut self) -> PResult<Expr> {
        let mut l = self.product()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Sym("+")) => BinOp::Add,
                Some(Tok::Sym("-")) => BinOp::Sub,
                _ => return Ok(l),
            };
            let line = self.line();
            self.pos += 1;
            let r = self.product()?;
            l = Expr::Bin {
                op,
                l: Box::new(l),
                r: Box::new(r),
                line,
            };
        }
    }
    fn product(&mut self) -> PResult<Expr> {
        let mut l = self.unary()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Sym("*")) => BinOp::Mul,
                Some(Tok::Sym("/")) => BinOp::Div,
                _ => return Ok(l),
            };
            let line = self.line();
            self.pos += 1;
            let r = self.unary()?;
            l = Expr::Bin {
                op,
                l: Box::new(l),
                r: Box::new(r),
                line,
            };
        }
    }
    fn unary(&mut self) -> PResult<Expr> {
        if self.is_sym("-") {
            self.pos += 1;
            return Ok(Expr::Neg(Box::new(self.unary()?)));
        }
        if self.is_sym("+") {
            self.pos += 1;
            return self.unary();
        }
        self.power()
    }
    fn power(&mut self) -> PResult<Expr> {
        let base = self.atom()?;
        if self.is_sym("^") {
            let line = self.line();
            self.pos += 1;
            let exp = self.unary()?;
            return Ok(Expr::Bin {
                op: BinOp::Pow,
                l: Box::new(base),
                r: Box::new(exp),
                line,
            });
        }
        Ok(base)
    }
    fn unit_after(&mut self) -> PResult<Option<(f64, Dim)>> {
        if let Some(Tok::Unit(u)) = self.peek().cloned() {
            let line = self.line();
            self.pos += 1;
            return parse_unit(&u).map(Some).map_err(|e| Diag::err(line, e));
        }
        Ok(None)
    }
    fn atom(&mut self) -> PResult<Expr> {
        let line = self.line();
        match self.next() {
            Some(Tok::Num(v)) => {
                let unit = self.unit_after()?;
                let (f, dim) = unit.unwrap_or((1.0, Dim::NONE));
                Ok(Expr::Num {
                    si: v * f,
                    dim,
                    bare: unit.is_none(),
                    line,
                })
            }
            Some(Tok::Ident(n)) if n == "true" || n == "false" => Ok(Expr::Bool {
                value: n == "true",
                line,
            }),
            Some(Tok::Ident(n)) if KEYWORDS.contains(&n.as_str()) => Err(Diag::err(
                line,
                format!("«{n}» cannot stand in an expression"),
            )),
            Some(Tok::Ident(n)) => {
                if self.is_sym("(") {
                    self.pos += 1;
                    let mut args = Vec::new();
                    if !self.is_sym(")") {
                        loop {
                            args.push(self.expr()?);
                            if self.is_sym(",") {
                                self.pos += 1;
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect_sym(")")?;
                    Ok(Expr::Call {
                        name: n,
                        args,
                        line,
                    })
                } else {
                    Ok(Expr::Var { name: n, line })
                }
            }
            Some(Tok::Sym("(")) => {
                let e = self.expr()?;
                self.expect_sym(")")?;
                Ok(e)
            }
            Some(Tok::Sym("[")) => {
                let mut vals = Vec::new();
                loop {
                    let neg = if self.is_sym("-") {
                        self.pos += 1;
                        true
                    } else {
                        false
                    };
                    match self.next() {
                        Some(Tok::Num(v)) => vals.push(if neg { -v } else { v }),
                        _ => return Err(Diag::err(
                            line,
                            "a table holds plain numbers, with its unit after the closing bracket",
                        )),
                    }
                    if self.is_sym(",") {
                        self.pos += 1;
                        continue;
                    }
                    break;
                }
                self.expect_sym("]")?;
                let (f, dim) = self.unit_after()?.unwrap_or((1.0, Dim::NONE));
                Ok(Expr::Table {
                    si: vals.into_iter().map(|v| v * f).collect(),
                    dim,
                    line,
                })
            }
            Some(Tok::Unit(u)) => Err(Diag::err(
                line,
                format!("a unit [{u}] belongs straight after a number"),
            )),
            _ => Err(Diag::err(line, "expected a value here")),
        }
    }
}

/// Read a method. Only the grammar is checked here; [`check`] does the rest.
pub fn parse(src: &str) -> Result<Program, Diag> {
    let toks = lex(src)?;
    let mut p = Parser { toks, pos: 0 };
    let body = p.block(&[])?;
    if body.is_empty() {
        return Err(Diag::err(0, "the method is empty"));
    }
    Ok(Program { body })
}
