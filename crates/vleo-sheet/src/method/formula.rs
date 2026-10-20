//! The method language: a formula as a person types it, read into a method
//! (docs/briefs/F8.md, F8-a, item 3).
//!
//! One line `NAME = EXPR`, a bare expression, or several lines, the last the
//! answer and each before it a `let`. Each expression is the language's own:
//! its operators, functions, constants and units, parsed and checked by
//! [`compile`] against the node's [`Signature`]. A spelling with one meaning
//! is written back in the language and said: `**` as `^`, `×` and `·` as
//! `*`, `−` as `-` (F8's question 59); anything else the language does not
//! read is refused, the character named, never guessed.
//!
//! Every name the formula reads that is not one it works out first, a
//! kernel constant or a function is an input, found among the node's input
//! ports by name, else by symbol, written then as a `let` alias (question
//! 62). A name found neither way is refused and listed as missing; a near
//! name is offered, never taken. Inputs come in as dimensioned values in
//! their ports' units; for an empirical fit, whose numbers hold only in
//! named units, the person names the unit of each input and of the answer,
//! and the method reads each input as a number in it (question 69). The
//! program never chooses a unit.
//!
//! Nothing runs here: whether the method agrees with the node's cases is the
//! interpreter's to say, on the cases, as for any method.

use super::*;

/// One of the node's input ports, as the formula may name it.
#[derive(Clone, Debug, PartialEq)]
pub struct Port {
    pub name: String,
    /// The symbol a method writes it as, when the port has one.
    pub symbol: String,
    pub unit: String,
}

/// The units an empirical fit holds in, named by the person: each input's,
/// by the name the formula reads it by or its port's, and the answer's.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fit {
    pub inputs: Vec<(String, String)>,
    pub answer: String,
}

/// How an input the formula reads was found among the ports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Found {
    Name,
    Symbol,
    Missing,
}

/// An input the formula reads.
#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    /// As the formula writes it.
    pub name: String,
    /// The port it is, when it was found.
    pub port: Option<String>,
    pub found: Found,
    /// The port's unit, or the fit's, as the method reads it.
    pub unit: String,
    /// A near name, offered and never taken.
    pub offered: Option<String>,
}

/// A formula read: the method text it was written as, once every input is
/// found and every unit named (whether the checker holds it sound is
/// [`Read::ok`]: text that does not compile is kept to be shown, never to be
/// saved), each input found, each spelling written back, a note, and every
/// finding, by line of the formula (0 for the formula as a whole).
#[derive(Clone, Debug, PartialEq)]
pub struct Read {
    pub typed: String,
    pub method: Option<String>,
    pub inputs: Vec<Input>,
    pub rewrites: Vec<String>,
    pub note: Option<String>,
    pub diags: Vec<Diag>,
}

impl Read {
    /// Whether it became a method the checker holds sound.
    pub fn ok(&self) -> bool {
        self.method.is_some() && !self.diags.iter().any(|d| d.severity == Severity::Error)
    }
}

/// One piece of an expression, as the scanner sees it.
#[derive(Clone, Debug, PartialEq)]
enum Piece {
    Num,
    Var(String),
    Call(String),
    Open,
    Close,
    Op,
}

/// The pieces of `expr`, with where each starts and ends, or what the
/// language does not read in it.
fn pieces(expr: &str) -> Result<Vec<(Piece, usize, usize)>, String> {
    let b = expr.as_bytes();
    let mut out = Vec::new();
    let mut p = 0;
    while p < b.len() {
        let c = b[p];
        if c.is_ascii_whitespace() {
            p += 1;
            continue;
        }
        let start = p;
        if c.is_ascii_digit() || (c == b'.' && p + 1 < b.len() && b[p + 1].is_ascii_digit()) {
            while p < b.len() && (b[p].is_ascii_digit() || b[p] == b'.') {
                p += 1;
            }
            if p < b.len() && (b[p] == b'e' || b[p] == b'E') {
                let save = p;
                p += 1;
                if p < b.len() && (b[p] == b'+' || b[p] == b'-') {
                    p += 1;
                }
                if p < b.len() && b[p].is_ascii_digit() {
                    while p < b.len() && b[p].is_ascii_digit() {
                        p += 1;
                    }
                } else {
                    p = save;
                }
            }
            // A unit in brackets after the number is the number's.
            let mut q = p;
            while q < b.len() && b[q].is_ascii_whitespace() {
                q += 1;
            }
            if q < b.len() && b[q] == b'[' {
                let Some(close) = expr[q..].find(']') else {
                    return Err(format!(
                        "the unit after «{}» is not closed with ]",
                        &expr[start..p]
                    ));
                };
                p = q + close + 1;
            }
            out.push((Piece::Num, start, p));
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            while p < b.len() && (b[p].is_ascii_alphanumeric() || b[p] == b'_') {
                p += 1;
            }
            let name = expr[start..p].to_string();
            let mut q = p;
            while q < b.len() && b[q].is_ascii_whitespace() {
                q += 1;
            }
            let called = q < b.len() && b[q] == b'(';
            let is_fn = function(&name).is_some() || kernel_function(&name).is_some();
            if called && !is_fn {
                let lower = name.to_ascii_lowercase();
                return Err(
                    if function(&lower).is_some() || kernel_function(&lower).is_some() {
                        format!("«{name}» is not a function of the method language: did you mean «{lower}»?")
                    } else {
                        format!(
                            "«{name}» is not a function of the method language: write * for a product, as {name} * (…)"
                        )
                    },
                );
            }
            out.push((
                if called {
                    Piece::Call(name)
                } else {
                    Piece::Var(name)
                },
                start,
                p,
            ));
            continue;
        }
        let piece = match c {
            b'(' => Piece::Open,
            b')' => Piece::Close,
            b'+' | b'-' | b'*' | b'/' | b'^' | b',' | b'<' | b'>' | b'!' | b'=' => Piece::Op,
            b'#' => {
                return Err(
                    "a comment is not part of a formula: once it is the method, a comment may follow it there"
                        .into(),
                )
            }
            b'[' => {
                return Err(
                    "«[» here: a unit is written straight after its number, as 6371 [km]; a list is written in a method"
                        .into(),
                )
            }
            b':' | b'"' => {
                return Err(format!(
                    "«{}» is written in a method, not in a formula",
                    c as char
                ))
            }
            _ => {
                return Err(format!(
                    "«{}» is not an operator of the method language: + - * / ^",
                    c as char
                ));
            }
        };
        p += 1;
        out.push((piece, start, p));
    }
    // Two values side by side are a product written without its *.
    for w in out.windows(2) {
        let ends = matches!(w[0].0, Piece::Num | Piece::Var(_) | Piece::Close);
        let starts = matches!(
            w[1].0,
            Piece::Num | Piece::Var(_) | Piece::Call(_) | Piece::Open
        );
        let words = matches!(&w[0].0, Piece::Var(n) if ["and", "or", "not"].contains(&n.as_str()))
            || matches!(&w[1].0, Piece::Var(n) if ["and", "or", "not"].contains(&n.as_str()));
        if ends && starts && !words {
            return Err(format!(
                "«{}» and «{}» stand side by side: write * between them for a product",
                &expr[w[0].1..w[0].2],
                &expr[w[1].1..w[1].2]
            ));
        }
    }
    Ok(out)
}

/// `expr` with each variable named in `rename` written as its new name.
fn renamed(
    expr: &str,
    found: &[(Piece, usize, usize)],
    rename: &BTreeMap<String, String>,
) -> String {
    let mut out = String::new();
    let mut at = 0;
    for (piece, s, e) in found {
        if let Piece::Var(n) = piece {
            if let Some(to) = rename.get(n) {
                out.push_str(&expr[at..*s]);
                out.push_str(to);
                at = *e;
            }
        }
    }
    out.push_str(&expr[at..]);
    out
}

/// The `=` that assigns, if the line has one: not part of `==`, `<=`, `>=`
/// or `!=`.
fn assigns(line: &str) -> Option<usize> {
    let b = line.as_bytes();
    (0..b.len()).find(|&i| {
        b[i] == b'='
            && (i == 0 || !b"<>!=".contains(&b[i - 1]))
            && (i + 1 >= b.len() || b[i + 1] != b'=')
    })
}

fn is_name(s: &str) -> bool {
    let mut c = s.chars();
    c.next()
        .is_some_and(|f| f.is_ascii_alphabetic() || f == '_')
        && c.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The spellings with one meaning, written back in the language: what was
/// typed and what it is written as.
const SPELLINGS: &[(&str, &str)] = &[("**", "^"), ("×", "*"), ("·", "*"), ("−", "-")];

/// Read `typed`, a formula, into a method for a node whose inputs are
/// `ports` (in `sig`'s order) and whose output is `output`, checked against
/// `sig`; with `fit`, an empirical fit in the units it names.
pub fn read(
    typed: &str,
    sig: &Signature,
    ports: &[Port],
    output: &Port,
    fit: Option<&Fit>,
) -> Read {
    let mut r = Read {
        typed: typed.to_string(),
        method: None,
        inputs: Vec::new(),
        rewrites: Vec::new(),
        note: None,
        diags: Vec::new(),
    };
    let mut text = typed.to_string();
    for (from, to) in SPELLINGS {
        if text.contains(from) {
            text = text.replace(from, to);
            r.rewrites.push(format!("«{from}» is written «{to}»"));
        }
    }
    // The lines of the formula, each with its line as typed.
    let lines: Vec<(usize, &str)> = text
        .lines()
        .enumerate()
        .map(|(i, l)| (i + 1, l.trim()))
        .filter(|(_, l)| !l.is_empty())
        .collect();
    if lines.is_empty() {
        r.diags.push(Diag::err(
            0,
            format!(
                "nothing is typed: write the formula, as {} = 0.5 * rho * v^2 * cd * a",
                output.name
            ),
        ));
        return r;
    }
    for (n, l) in &lines {
        if let Some(c) = l.chars().find(|c| !c.is_ascii()) {
            let fix = if "⁰¹²³⁴⁵⁶⁷⁸⁹".contains(c) {
                "write a power with ^, as v^2".to_string()
            } else if c.is_alphabetic() {
                format!(
                    "a name is written in ASCII letters, as the node's ports name it: {}",
                    ports
                        .iter()
                        .map(|p| p.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            } else {
                "it is no operator of the method language: + - * / ^".to_string()
            };
            r.diags.push(Diag::err(
                *n,
                format!("«{c}» is not part of the method language: {fix}"),
            ));
        }
    }
    if !r.diags.is_empty() {
        return r;
    }
    // Each line: what it works out, if it names it, and its expression.
    let mut steps: Vec<(usize, Option<String>, String)> = Vec::new();
    for (i, (n, l)) in lines.iter().enumerate() {
        let last = i + 1 == lines.len();
        let (lhs, rhs) = match assigns(l) {
            Some(at) => (
                Some(l[..at].trim().to_string()),
                l[at + 1..].trim().to_string(),
            ),
            None => (None, l.to_string()),
        };
        match &lhs {
            Some(name) if !is_name(name) => r.diags.push(Diag::err(
                *n,
                format!("«{name}» is not a name: what a line works out is one name, as q"),
            )),
            Some(name) if KEYWORDS.contains(&name.as_str()) => r.diags.push(Diag::err(
                *n,
                format!("«{name}» is a word of the method language, not a name"),
            )),
            None if !last => r.diags.push(Diag::err(
                *n,
                "each line before the last names what it works out, as q = 0.5 * rho * v^2",
            )),
            // A line before the last works out a value of its own: never one
            // of the node's inputs, nor a kernel constant, under its name.
            Some(name) if !last => {
                if let Some(p) = ports
                    .iter()
                    .find(|p| &p.name == name || (!p.symbol.is_empty() && &p.symbol == name))
                {
                    r.diags.push(Diag::err(
                        *n,
                        format!(
                            "«{name}» is the input {}: a line works out a name of its own, as q",
                            p.name
                        ),
                    ));
                } else if kernel_constant(name).is_some() {
                    r.diags.push(Diag::err(
                        *n,
                        format!("«{name}» is a kernel constant: a line works out a name of its own, as q"),
                    ));
                }
            }
            _ => {}
        }
        if rhs.is_empty() {
            r.diags.push(Diag::err(
                *n,
                "nothing follows the =: write what it works out",
            ));
        }
        steps.push((*n, lhs, rhs));
    }
    if !r.diags.is_empty() {
        return r;
    }
    // The names each expression reads, and the inputs among them.
    let mut scanned: Vec<Vec<(Piece, usize, usize)>> = Vec::new();
    let mut local: BTreeSet<String> = BTreeSet::new();
    let worked_out: BTreeSet<String> = steps[..steps.len() - 1]
        .iter()
        .filter_map(|s| s.1.clone())
        .collect();
    for (i, (n, lhs, rhs)) in steps.iter().enumerate() {
        match pieces(rhs) {
            Ok(p) => {
                for (piece, ..) in &p {
                    let Piece::Var(name) = piece else { continue };
                    if local.contains(name)
                        || ["and", "or", "not", "true", "false"].contains(&name.as_str())
                        || r.inputs.iter().any(|x| &x.name == name)
                    {
                        continue;
                    }
                    if kernel_constant(name).is_some() {
                        // A constant is the constant; a port that writes it
                        // as its symbol is never taken for it unsaid.
                        if let Some(p) = ports.iter().find(|p| &p.symbol == name) {
                            r.diags.push(Diag::err(
                                *n,
                                format!(
                                    "«{name}» is a kernel constant and the symbol of the input {}: write {} for the input",
                                    p.name, p.name
                                ),
                            ));
                        }
                        continue;
                    }
                    if KEYWORDS.contains(&name.as_str()) {
                        r.diags.push(Diag::err(
                            *n,
                            format!("«{name}» is a word of the method language: a formula is one expression a line"),
                        ));
                        continue;
                    }
                    if worked_out.contains(name) {
                        r.diags.push(Diag::err(
                            *n,
                            format!("«{name}» is read before the line that works it out: write that line first"),
                        ));
                        continue;
                    }
                    let by_name = ports.iter().find(|p| &p.name == name);
                    let by_symbol = ports
                        .iter()
                        .find(|p| !p.symbol.is_empty() && &p.symbol == name);
                    let (port, found) = match (by_name, by_symbol) {
                        (Some(p), _) => (Some(p), Found::Name),
                        (None, Some(p)) => (Some(p), Found::Symbol),
                        (None, None) => (None, Found::Missing),
                    };
                    let offered = match port {
                        Some(_) => None,
                        None => KERNEL_CONSTANTS
                            .iter()
                            .find(|c| c.name.eq_ignore_ascii_case(name))
                            .map(|c| c.name.to_string())
                            .or_else(|| {
                                ports.iter().find_map(|p| {
                                    if p.name.eq_ignore_ascii_case(name) {
                                        Some(p.name.clone())
                                    } else if !p.symbol.is_empty()
                                        && p.symbol.eq_ignore_ascii_case(name)
                                    {
                                        Some(p.symbol.clone())
                                    } else {
                                        None
                                    }
                                })
                            }),
                    };
                    r.inputs.push(Input {
                        name: name.clone(),
                        port: port.map(|p| p.name.clone()),
                        found,
                        unit: port.map_or(String::new(), |p| p.unit.clone()),
                        offered,
                    });
                }
                scanned.push(p);
            }
            Err(why) => {
                r.diags.push(Diag::err(*n, why));
                scanned.push(Vec::new());
            }
        }
        if i + 1 < steps.len() {
            if let Some(name) = lhs {
                local.insert(name.clone());
            }
        }
    }
    let missing: Vec<String> = r
        .inputs
        .iter()
        .filter(|x| x.found == Found::Missing)
        .map(|x| match &x.offered {
            Some(o) => format!("«{}» (did you mean «{o}»?)", x.name),
            None => format!("«{}»", x.name),
        })
        .collect();
    if !missing.is_empty() {
        r.diags.push(Diag::err(
            0,
            format!(
                "the formula reads {}, which {} none of this node's inputs ({}) by name or symbol: ask for {} as {}, or write the name its port has",
                missing.join(", "),
                if missing.len() == 1 { "is" } else { "are" },
                ports.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", "),
                if missing.len() == 1 { "it" } else { "them" },
                if missing.len() == 1 { "an input" } else { "inputs" },
            ),
        ));
    }
    if !r.diags.is_empty() {
        return r;
    }
    // What the answer is named, beside the output's own name.
    if let Some(Some(name)) = steps.last().map(|s| s.1.as_ref()) {
        if name != &output.name && name != &output.symbol {
            r.note = Some(format!(
                "the formula names its answer «{name}», and the node's output is «{}»: it is returned as {}",
                output.name, output.name
            ));
        }
    }
    // The method, line by line: the formula as typed, then the aliases and
    // units, then the formula.
    let mut method: Vec<String> = typed
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| format!("# Typed as: {}", l.trim()))
        .collect();
    let mut rename: BTreeMap<String, String> = BTreeMap::new();
    let mut answer_unit: Option<String> = None;
    match fit {
        None => {
            for x in r.inputs.iter().filter(|x| x.found == Found::Symbol) {
                method.push(format!(
                    "let {} = {}",
                    x.name,
                    x.port.as_deref().unwrap_or("")
                ));
            }
        }
        Some(fit) => {
            let dim_of = |port: &str| sig.inputs.iter().find(|(n, _)| n == port).map(|(_, d)| *d);
            // Every unit named is of an input the formula reads, and each
            // input is named once.
            for (named, _) in &fit.inputs {
                let of: Vec<&Input> = r
                    .inputs
                    .iter()
                    .filter(|x| &x.name == named || x.port.as_deref() == Some(named.as_str()))
                    .collect();
                if of.is_empty() {
                    r.diags.push(Diag::err(
                        0,
                        format!("a unit is named for «{named}», which the formula does not read"),
                    ));
                }
            }
            for x in &r.inputs {
                let port = x.port.as_deref().unwrap_or("");
                let times = fit
                    .inputs
                    .iter()
                    .filter(|(n, _)| n == &x.name || n == port)
                    .count();
                if times > 1 {
                    r.diags.push(Diag::err(
                        0,
                        format!("«{}» is given a unit {times} times: name it once", x.name),
                    ));
                }
            }
            for x in &mut r.inputs {
                let port = x.port.clone().unwrap_or_default();
                let named = fit
                    .inputs
                    .iter()
                    .find(|(n, _)| n == &x.name || n == &port)
                    .map(|(_, u)| u.trim().to_string());
                let Some(unit) = named.filter(|u| !u.is_empty()) else {
                    r.diags.push(Diag::err(
                        0,
                        format!("an empirical fit holds in the units named: name the unit «{}» is read in", x.name),
                    ));
                    continue;
                };
                match (parse_unit(&unit), dim_of(&port)) {
                    (Err(e), _) => r.diags.push(Diag::err(
                        0,
                        format!("[{unit}] for «{}»: {}", x.name, e.message()),
                    )),
                    (Ok((_, d)), Some(want)) if d != want => r.diags.push(Diag::err(
                        0,
                        format!(
                            "[{unit}] is {}, and «{}» is {}",
                            super::amount::kind_of(d),
                            x.name,
                            super::amount::kind_of(want)
                        ),
                    )),
                    _ => {
                        let alias = if x.name == port {
                            format!("{}_number", x.name)
                        } else {
                            x.name.clone()
                        };
                        method.push(format!("let {alias} = {port} / 1 [{unit}]"));
                        if alias != x.name {
                            rename.insert(x.name.clone(), alias);
                        }
                        x.unit = unit;
                    }
                }
            }
            let answer = fit.answer.trim();
            match parse_unit(answer) {
                _ if answer.is_empty() => r.diags.push(Diag::err(
                    0,
                    "an empirical fit holds in the units named: name the unit its answer is in",
                )),
                Err(e) => r.diags.push(Diag::err(
                    0,
                    format!("[{answer}] for the answer: {}", e.message()),
                )),
                Ok((_, d)) if d != sig.output => r.diags.push(Diag::err(
                    0,
                    format!(
                        "[{answer}] is {}, and the node's output is {}",
                        super::amount::kind_of(d),
                        super::amount::kind_of(sig.output)
                    ),
                )),
                Ok(_) => answer_unit = Some(answer.to_string()),
            }
            if !r.diags.is_empty() {
                return r;
            }
        }
    }
    let head = method.len();
    for (i, ((_, lhs, rhs), found)) in steps.iter().zip(&scanned).enumerate() {
        let expr = renamed(rhs, found, &rename);
        if i + 1 < steps.len() {
            method.push(format!("let {} = {expr}", lhs.as_deref().unwrap_or("")));
        } else {
            method.push(match &answer_unit {
                Some(u) => format!("return ({expr}) * 1 [{u}]"),
                None => format!("return {expr}"),
            });
        }
    }
    let text = method.join("\n") + "\n";
    // The checker's findings, on the formula's lines.
    let at = |line: usize| -> usize {
        if line > head {
            steps.get(line - head - 1).map_or(0, |s| s.0)
        } else {
            0
        }
    };
    match compile(&text, sig) {
        Ok(p) => {
            r.diags = check(&p, sig)
                .into_iter()
                .map(|d| Diag {
                    line: at(d.line),
                    ..d
                })
                .collect();
        }
        Err(diags) => {
            r.diags = diags
                .into_iter()
                .map(|d| Diag {
                    line: at(d.line),
                    ..d
                })
                .collect();
        }
    }
    r.method = Some(text);
    r
}
