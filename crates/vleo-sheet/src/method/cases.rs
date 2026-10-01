//! The method language: the author's cases, against the method.

use super::*;

/// One of the author's test cases, in SI: the inputs by binding name, and
/// either the answer their own code gave or the fact that it must refuse.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Case {
    pub label: String,
    pub inputs: Vec<(String, f64)>,
    /// `None` for a case that must be refused.
    pub expect: Option<f64>,
    /// Relative, exactly as a fixture's: |got − expect| / |expect|, or |got|
    /// when the expected value is zero.
    pub tolerance: f64,
}

impl Case {
    pub fn refuses(&self) -> bool {
        self.expect.is_none()
    }
}

/// How many normal cases, and how many refusals, a node's author supplies at
/// least. Three answers, so one wrong case cannot be the only evidence; one
/// refusal, so the method is shown to say no where the author's code says no
/// — a method that answers everything is a method nobody has tried to break.
pub const MIN_CASES: usize = 3;
pub const MIN_REFUSALS: usize = 1;

#[derive(Clone, Debug, PartialEq)]
pub enum Verdict {
    /// Within tolerance, or refused as the author said it must be.
    Agrees,
    /// Answered, and further from the author's value than the tolerance.
    Differs { got: f64, relative: f64 },
    /// Answered where the author's code refused.
    AnsweredRefusal { got: f64 },
    /// Refused where the author's code answered.
    RefusedAnswer { reason: String },
    /// The method faulted: a square root of a negative number, say.
    Faulted(String),
    /// The case names an input the node does not have, or leaves one out.
    Malformed(String),
}

impl Verdict {
    pub fn agrees(&self) -> bool {
        *self == Verdict::Agrees
    }
    /// One line, for a terminal, the gate and the form.
    pub fn text(&self, c: &Case) -> String {
        match self {
            Verdict::Agrees if c.refuses() => "refused, as your code does".into(),
            Verdict::Agrees => "agrees with your code".into(),
            Verdict::Differs { got, relative } => format!(
                "the method gives {}, your code gave {} — {:.2e} apart (relative), more than your tolerance {:e}",
                show(*got),
                show(c.expect.unwrap_or(0.0)),
                relative,
                c.tolerance
            ),
            Verdict::AnsweredRefusal { got } => format!(
                "your code refuses this case, but the method answers {}",
                show(*got)
            ),
            Verdict::RefusedAnswer { reason } => format!(
                "your code answers this case, but the method refuses it: {reason}"
            ),
            Verdict::Faulted(m) => format!("the method faults here: {m}"),
            Verdict::Malformed(m) => m.clone(),
        }
    }
}

/// A number as a person reads it: plainly where that is short, in powers of
/// ten where it is not.
pub fn show(v: f64) -> String {
    let a = v.abs();
    if a == 0.0 || (1e-4..1e9).contains(&a) {
        format!("{v}")
    } else {
        format!("{v:e}")
    }
}

fn relative_error(got: f64, expected: f64) -> f64 {
    if expected == 0.0 {
        pmath::abs(got)
    } else {
        pmath::abs((got - expected) / expected)
    }
}

/// What the method gave for one case, when it gave a number.
pub fn value_of(p: &Program, c: &Case) -> Option<f64> {
    match run(p, &c.inputs) {
        Ok(Outcome::Answer(v)) => Some(v),
        _ => None,
    }
}

/// Run one case and say whether the method agrees with the author's code.
pub fn judge(p: &Program, sig: &Signature, c: &Case) -> Verdict {
    for (name, _) in &sig.inputs {
        if !c.inputs.iter().any(|(n, _)| n == name) {
            return Verdict::Malformed(format!("this case gives no value for the input «{name}»"));
        }
    }
    for (name, _) in &c.inputs {
        if !sig.inputs.iter().any(|(n, _)| n == name) {
            return Verdict::Malformed(format!("«{name}» is not an input of this node"));
        }
    }
    match (run(p, &c.inputs), c.expect) {
        (Err(d), _) => Verdict::Faulted(d.to_string()),
        (Ok(Outcome::Refused { .. }), None) => Verdict::Agrees,
        (Ok(Outcome::Refused { reason, .. }), Some(_)) => Verdict::RefusedAnswer { reason },
        (Ok(Outcome::Answer(got)), None) => Verdict::AnsweredRefusal { got },
        (Ok(Outcome::Answer(got)), Some(want)) => {
            let relative = relative_error(got, want);
            if relative <= c.tolerance {
                Verdict::Agrees
            } else {
                Verdict::Differs { got, relative }
            }
        }
    }
}

/// Everything the checker has to say about one node's method and cases.
#[derive(Clone, Debug, Default)]
pub struct Report {
    /// The method's own findings: grammar, names, units, paths.
    pub diags: Vec<Diag>,
    /// Each case with its verdict, in the order given. Empty when the method
    /// has errors, since nothing can be run.
    pub cases: Vec<(Case, Verdict)>,
    /// What the method gave for each case, when it gave a number, so a face
    /// can show it beside the author's value.
    pub got: Vec<Option<f64>>,
    /// What is missing from the set of cases: too few answers, no refusal.
    pub shortfall: Vec<String>,
}

impl Report {
    /// Sound: the method checks, every case agrees, and the set is complete.
    pub fn sound(&self) -> bool {
        !self.diags.iter().any(|d| d.severity == Severity::Error)
            && self.cases.iter().all(|(_, v)| v.agrees())
            && self.shortfall.is_empty()
    }

    /// As JSON, for the form and the web face.
    pub fn json(&self) -> String {
        let q = jstr;
        let mut o = String::from("{");
        let _ = write!(
            o,
            "\"sound\": {}, \"version\": {LANGUAGE_VERSION}, \"diags\": [",
            self.sound()
        );
        for (i, d) in self.diags.iter().enumerate() {
            let _ = write!(
                o,
                "{}{{\"line\": {}, \"severity\": {}, \"msg\": {}}}",
                if i > 0 { ", " } else { "" },
                d.line,
                q(match d.severity {
                    Severity::Error => "error",
                    Severity::Warning => "note",
                }),
                q(&d.msg)
            );
        }
        o.push_str("], \"cases\": [");
        for (i, (c, v)) in self.cases.iter().enumerate() {
            let got = match v {
                Verdict::Differs { got, .. } | Verdict::AnsweredRefusal { got } => {
                    format!("{got:e}")
                }
                _ => "null".into(),
            };
            let _ = write!(
                o,
                "{}{{\"label\": {}, \"agrees\": {}, \"refuse\": {}, \"got\": {got}, \"text\": {}}}",
                if i > 0 { ", " } else { "" },
                q(&c.label),
                v.agrees(),
                c.refuses(),
                q(&v.text(c))
            );
        }
        o.push_str("], \"shortfall\": [");
        for (i, s) in self.shortfall.iter().enumerate() {
            let _ = write!(o, "{}{}", if i > 0 { ", " } else { "" }, q(s));
        }
        o.push_str("]}");
        o
    }
}

/// A JSON string. Its own copy rather than the form module's, so the checker
/// the node form carries is built from this file and the units alone — see
/// [`CHECKER_SOURCES`].
fn jstr(v: &str) -> String {
    let mut o = String::with_capacity(v.len() + 2);
    o.push('"');
    for c in v.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(o, "\\u{:04x}", c as u32);
            }
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// The files the node form's checker is compiled from. `web/method.wasm.gz` is
/// built from these by `cargo run -p xtask -- method-wasm`, which records
/// their fingerprint beside it; a test refuses a checkout whose sources have
/// moved on from the checker every form carries.
pub const CHECKER_SOURCES: &[&str] = &[
    "crates/vleo-sheet/src/method.rs",
    "crates/vleo-sheet/src/method/ast.rs",
    "crates/vleo-sheet/src/method/cases.rs",
    "crates/vleo-sheet/src/method/check.rs",
    "crates/vleo-sheet/src/method/node.rs",
    "crates/vleo-sheet/src/method/parse.rs",
    "crates/vleo-sheet/src/method/reference.rs",
    "crates/vleo-sheet/src/method/run.rs",
    "crates/vleo-sheet/src/method/rust.rs",
    "crates/vleo-sheet/src/method/spec.rs",
    "crates/vleo-sheet/src/method/tests.rs",
    "crates/vleo-sheet/src/method/units.rs",
    "crates/vleo-sheet/src/lesson.rs",
    "crates/vleo-units/src/pmath.rs",
    "crates/vleo-units/src/method_rt.rs",
    "crates/vleo-units/src/unit.rs",
    "crates/vleo-units/src/quantity.rs",
    "crates/vleo-units/src/constants.rs",
    "crates/vleo-method-wasm/src/lib.rs",
    "crates/vleo-method-wasm/Cargo.toml",
];

/// The fingerprint of [`CHECKER_SOURCES`] in a checkout: line endings made
/// one kind first, so a Windows checkout agrees with Linux.
pub fn checker_fingerprint(root: &std::path::Path) -> Result<String, String> {
    let mut all = String::new();
    for f in CHECKER_SOURCES {
        let t = std::fs::read_to_string(root.join(f)).map_err(|e| format!("{f}: {e}"))?;
        all.push_str(f);
        all.push('\n');
        all.push_str(&t.replace("\r\n", "\n"));
    }
    Ok(format!("{:016x}", crate::fnv1a(&all)))
}

/// A report that could not be made, as JSON of the same shape.
pub fn error_json(msg: &str) -> String {
    format!(
        "{{\"sound\": false, \"version\": {LANGUAGE_VERSION}, \"error\": {}, \"diags\": [], \"cases\": [], \"shortfall\": []}}",
        jstr(msg)
    )
}

/// Check a method and run the author's cases through it.
pub fn report(src: &str, sig: &Signature, cases: &[Case]) -> Report {
    let mut r = Report::default();
    let normal = cases.iter().filter(|c| !c.refuses()).count();
    let refusals = cases.len() - normal;
    if normal < MIN_CASES {
        r.shortfall.push(format!(
            "{normal} case(s) with an answer — at least {MIN_CASES}, so no single case is the only evidence"
        ));
    }
    if refusals < MIN_REFUSALS {
        r.shortfall.push(format!(
            "no case that must be refused — at least {MIN_REFUSALS}, to show the method says no where your code does"
        ));
    }
    for c in cases {
        let tolerance_ok = c.tolerance > 0.0 && c.tolerance.is_finite();
        if !c.refuses() && !tolerance_ok {
            r.shortfall.push(format!(
                "«{}» has no tolerance above zero — say how close is close enough",
                c.label
            ));
        }
    }
    let p = match parse(src) {
        Ok(p) => p,
        Err(d) => {
            r.diags.push(d);
            return r;
        }
    };
    r.diags = check(&p, sig);
    if r.diags.iter().any(|d| d.severity == Severity::Error) {
        return r;
    }
    for c in cases {
        r.cases.push((c.clone(), judge(&p, sig, c)));
        r.got.push(value_of(&p, c));
    }
    r
}

/// The same, from a sheet's own text: `[method] text`, `[output] type`, every
/// `[[input]]` and every `[[case]]`. This is what the node form sends from the
/// browser, through WebAssembly, and what intake and the gate read from the
/// file — one function, so the three cannot disagree.
pub fn report_toml(text: &str) -> Result<Report, String> {
    let v: toml::Value = text.parse().map_err(|e| format!("not TOML: {e}"))?;
    let src = v
        .get("method")
        .and_then(|m| m.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("");
    let out = v
        .get("output")
        .and_then(|o| o.get("type"))
        .and_then(|t| t.as_str())
        .unwrap_or("");
    let output = quantity_dim(out)
        .ok_or_else(|| format!("the answer's quantity «{out}» is not one this tool has"))?;
    let mut inputs = Vec::new();
    for i in v
        .get("input")
        .and_then(|a| a.as_array())
        .into_iter()
        .flatten()
    {
        let b = i.get("binding").and_then(|x| x.as_str()).unwrap_or("");
        let t = i.get("type").and_then(|x| x.as_str()).unwrap_or("");
        let d = quantity_dim(t)
            .ok_or_else(|| format!("the input «{b}» has no known quantity «{t}»"))?;
        inputs.push((b.to_string(), d));
    }
    let cases = cases_of(&v)?;
    Ok(report(src, &Signature { inputs, output }, &cases))
}

/// The same report from the plain form the node form's checker is handed —
/// no TOML parser, so the WebAssembly every form carries does not need one.
///
/// ```text
/// output Velocity
/// input r Length
/// case 0|1 <expect> <tolerance> <name=value;name=value> <label …>
/// method
/// <the method, to the end>
/// ```
///
/// `case 1` must be refused; its expect and tolerance are `-`.
pub fn report_plain(text: &str) -> Result<Report, String> {
    let mut output = None;
    let mut inputs = Vec::new();
    let mut cases = Vec::new();
    let mut lines = text.lines();
    let mut src = String::new();
    while let Some(l) = lines.next() {
        let mut w = l.splitn(2, ' ');
        match (w.next().unwrap_or(""), w.next().unwrap_or("").trim()) {
            ("output", q) => {
                output = Some(quantity_dim(q).ok_or_else(|| {
                    format!("the answer's quantity «{q}» is not one this tool has")
                })?)
            }
            ("input", rest) => {
                let (b, q) = rest.split_once(' ').unwrap_or((rest, ""));
                let d = quantity_dim(q.trim())
                    .ok_or_else(|| format!("the input «{b}» has no known quantity «{q}»"))?;
                inputs.push((b.to_string(), d));
            }
            ("case", rest) => {
                let f: Vec<&str> = rest.splitn(5, ' ').collect();
                if f.len() < 4 {
                    return Err(format!("a case line is short: «{l}»"));
                }
                let num = |x: &str| {
                    x.parse::<f64>()
                        .map_err(|_| format!("«{x}» is not a number"))
                };
                let refuse = f[0] == "1";
                let mut ins = Vec::new();
                for kv in f[3].split(';').filter(|x| !x.is_empty()) {
                    let (k, v) = kv
                        .split_once('=')
                        .ok_or_else(|| format!("«{kv}» is not name=value"))?;
                    ins.push((k.to_string(), num(v)?));
                }
                cases.push(Case {
                    label: f.get(4).unwrap_or(&"").to_string(),
                    inputs: ins,
                    expect: if refuse { None } else { Some(num(f[1])?) },
                    tolerance: if refuse { 0.0 } else { num(f[2])? },
                });
            }
            ("method", _) => {
                src = lines.collect::<Vec<_>>().join("\n");
                break;
            }
            ("", _) => {}
            (other, _) => return Err(format!("«{other}» is not a line the checker reads")),
        }
    }
    let output = output.ok_or("the answer's quantity is not given")?;
    Ok(report(&src, &Signature { inputs, output }, &cases))
}

/// The `[[case]]` blocks of a parsed sheet.
pub fn cases_of(v: &toml::Value) -> Result<Vec<Case>, String> {
    let mut out = Vec::new();
    for (i, c) in v
        .get("case")
        .and_then(|a| a.as_array())
        .into_iter()
        .flatten()
        .enumerate()
    {
        let num = |x: &toml::Value| x.as_float().or_else(|| x.as_integer().map(|n| n as f64));
        let label = c
            .get("label")
            .and_then(|x| x.as_str())
            .map(String::from)
            .unwrap_or_else(|| format!("case {}", i + 1));
        let refuse = c.get("refuse").and_then(|x| x.as_str()) == Some("yes");
        let expect = if refuse {
            None
        } else {
            Some(c.get("expect").and_then(num).ok_or_else(|| {
                format!("«{label}» has no expected value (or say refuse = \"yes\")")
            })?)
        };
        let mut inputs = Vec::new();
        if let Some(t) = c.get("inputs").and_then(|x| x.as_table()) {
            for (k, x) in t {
                let n =
                    num(x).ok_or_else(|| format!("«{label}»: the input «{k}» is not a number"))?;
                inputs.push((k.clone(), n));
            }
        }
        out.push(Case {
            label,
            inputs,
            expect,
            tolerance: c.get("tolerance").and_then(num).unwrap_or(0.0),
        });
    }
    Ok(out)
}
