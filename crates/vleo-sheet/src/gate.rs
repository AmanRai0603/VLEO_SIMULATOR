//! The gate, and the assembly validations.
//!
//! One gate binary, called from three places: the authoring hook, the pipeline,
//! and by hand on the test machine. Where the hook runs one script and the
//! pipeline runs another they drift within a month and the hook becomes
//! theatre, so there is one of them.
//!
//! The checks are ordered by cost, not by importance. The first seven run on
//! every edit in about a second; the rest run on demand and in the pipeline.

use crate::emit;
use crate::load::{read_holes, Tree};
use crate::model::*;
use std::collections::{BTreeMap, BTreeSet};

/// Format a candidate the way the generator does before writing it, so the
/// regeneration check asks "did the content drift" rather than "has the
/// formatter run".
///
/// When `rustfmt` is not on the path, or refuses the candidate, this returns
/// the text **unchanged**. It must never return anything but valid source: this
/// value is written to disk as well as compared, and an earlier version
/// returned a whitespace-collapsed form on failure — which put every generated
/// module on one line, where the first `//` comment swallowed the rest of the
/// file. The comparison is allowed to be weaker than the formatting; the
/// content is not allowed to be wrong.
pub fn formatted(text: &str) -> String {
    use std::io::Write;
    let dir = std::env::temp_dir().join("vleo-gate");
    let _ = std::fs::create_dir_all(&dir);
    let p = dir.join(format!("candidate-{}.rs", crate::fnv1a(text)));
    if let Ok(mut f) = std::fs::File::create(&p) {
        if f.write_all(text.as_bytes()).is_ok() {
            drop(f);
            // `--skip-children` because a generated `mod.rs` declares submodules
            // that do not exist beside a temporary file, and resolving them is
            // not what is being asked here.
            let ok = std::process::Command::new("rustfmt")
                .args(["--edition", "2021", "--quiet", "--skip-children"])
                .arg(&p)
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok {
                if let Ok(out) = std::fs::read_to_string(&p) {
                    let _ = std::fs::remove_file(&p);
                    return out;
                }
            }
            let _ = std::fs::remove_file(&p);
        }
    }
    text.to_string()
}

/// Collapse whitespace. The weaker comparison, used only when `rustfmt` is
/// absent.
fn normalise(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    /// Advisory. The gap pass reports what the sheet promised and nothing yet
    /// covers; its output is a list, not a verdict. It blocks the second human
    /// review rather than the build, because a tree that is 40% written should
    /// not have a red build every night — that is how a team learns to ignore
    /// one.
    Note(String),
    Fail(String),
}

#[derive(Clone, Debug)]
pub struct Check {
    pub name: &'static str,
    pub verdict: Verdict,
}

impl Check {
    fn pass(name: &'static str) -> Check {
        Check {
            name,
            verdict: Verdict::Pass,
        }
    }
    fn fail(name: &'static str, why: String) -> Check {
        Check {
            name,
            verdict: Verdict::Fail(why),
        }
    }
    fn note(name: &'static str, why: String) -> Check {
        Check {
            name,
            verdict: Verdict::Note(why),
        }
    }
    pub fn is_note(&self) -> bool {
        matches!(self.verdict, Verdict::Note(_))
    }
    pub fn failed(&self) -> bool {
        matches!(self.verdict, Verdict::Fail(_))
    }
}

/// The record of why the node is what it is — every version complete, in
/// order, with a reason. A version is a record, so a malformed one is refused
/// rather than noted: a de-risking narrative with holes in it reads as though
/// nothing was risked.
/// A lesson beside the sheet, if there is one: every key known, every claim
/// tagged and sourced where it says so, no markup, and every row a widget names
/// real — an input a person picks, an output that computes. `None` for a row
/// with no lesson, which is most of them and is not a gap.
fn lesson_check(sh: &Sheet, tree: &Tree) -> Option<Check> {
    Some(match crate::lesson::load(&sh.dir, &sh.id)? {
        Err(e) => Check::fail("lesson", e.into()),
        Ok(l) => {
            let bad = crate::lesson::problems(&l, tree);
            if bad.is_empty() {
                Check::pass("lesson")
            } else {
                Check::fail("lesson", bad.join("; "))
            }
        }
    })
}

fn versions_check(sh: &Sheet) -> Check {
    let bad = crate::derisk::version_problems(sh);
    if bad.is_empty() {
        Check::pass("versions")
    } else {
        Check::fail("versions", bad.join("; "))
    }
}

/// The method, the author's code and cases, and flight software.
///
/// Nothing here is asked of a row that has none of them: a node without a
/// method keeps its hand-written holes, and the gap pass lists the method as
/// still to come. What IS here is refused rather than noted, because each of
/// these is a claim — "this method is the relation", "these cases came from my
/// code" — and a claim that does not hold must not reach a release.
fn method_checks(sh: &Sheet) -> Vec<Check> {
    use crate::method;
    let mut out = Vec::new();
    let has_method = !sh.method.text.trim().is_empty();
    if has_method {
        // A row that publishes a set may have a method too: it publishes each
        // member with `publish`, and the checker holds every member published
        // before any answer (language version 3).
        let mut bad = Vec::new();
        if sh.is_declared() {
            bad.push("a declared row states a number; it has no method".into());
        }
        let who = sh.method.by.to_lowercase();
        if crate::form::agent_identities(&sh.dir).iter().any(|a| {
            who == *a || who.starts_with(&format!("{a} ")) || who.contains(&format!("{a}/"))
        }) {
            bad.push(format!(
                "the method is attributed to «{}», an assistant — an assistant may never supply \
                 mathematics",
                sh.method.by
            ));
        }
        let checker = sh.method.checked_by.to_lowercase();
        if !checker.is_empty()
            && crate::form::agent_identities(&sh.dir).iter().any(|a| {
                checker == *a
                    || checker.starts_with(&format!("{a} "))
                    || checker.contains(&format!("{a}/"))
            })
        {
            bad.push(format!(
                "the transcription is said to be checked by «{}», an assistant — a person reads \
                 the copy against its source",
                sh.method.checked_by
            ));
        }
        if !sh.method.checked_by.is_empty() && sh.method.transcribed_from.is_empty() {
            bad.push(
                "the method says who checked a transcription, and not what it was copied from"
                    .into(),
            );
        }
        let text = std::fs::read_to_string(sh.dir.join("node.toml")).unwrap_or_default();
        match method::report_toml(&text) {
            Err(e) => bad.push(e.into()),
            Ok(r) => {
                for d in r
                    .diags
                    .iter()
                    .filter(|d| d.severity == method::Severity::Error)
                {
                    bad.push(format!("method {d}"));
                }
                for (c, v) in &r.cases {
                    if !v.agrees() {
                        bad.push(format!("case «{}»: {}", c.label, v.text(c)));
                    }
                }
                // A method copied from the code it replaced is held to what
                // that code answered (baseline/transcribed.csv), not to author
                // cases it never had: cases made now would take their
                // expected values from the code under test (AGENTS.md, rule
                // 4). Its evidence is the fixtures it had before.
                if sh.method.transcribed_from.is_empty() {
                    bad.extend(r.shortfall.iter().cloned());
                }
            }
        }
        out.push(if bad.is_empty() {
            Check::pass("method")
        } else {
            Check::fail("method", bad.join("; "))
        });
    }
    // Cases say where they came from. One from the author's own code — or
    // that does not say, as every case once came from it — needs that code
    // here, to be read and run again; one worked by hand, in a spreadsheet or
    // from a paper does not.
    if !sh.cases.is_empty() {
        let mut bad = Vec::new();
        let from_code = sh
            .cases
            .iter()
            .any(|c| c.origin.is_empty() || c.origin == "code");
        if from_code && sh.author.code.trim().is_empty() {
            bad.push(
                "the cases came from your code, and the code is not here — paste it under \
                      [author] code"
                    .to_string(),
            );
        }
        if from_code && sh.author.language.trim().is_empty() {
            bad.push("say what language the code is in".into());
        }
        if from_code && sh.author.name.trim().is_empty() {
            bad.push("say who wrote the code".into());
        }
        if from_code && sh.author.test_code.trim().is_empty() {
            bad.push("the test code that ran the cases is not here".into());
        }
        for c in &sh.cases {
            for (k, _) in &c.inputs {
                if !sh.inputs.iter().any(|i| &i.binding == k) {
                    bad.push(format!(
                        "case «{}» sets «{k}», which is not an input of this row",
                        c.label
                    ));
                }
            }
        }
        out.push(if bad.is_empty() {
            Check::pass("cases")
        } else {
            Check::fail("cases", bad.join("; "))
        });
    }
    if !sh.flight.is_empty() {
        let mut bad = Vec::new();
        let mut names = BTreeSet::new();
        for f in &sh.flight {
            if f.name.trim().is_empty() || f.code.trim().is_empty() {
                bad.push("a flight software block needs its file name and its code".to_string());
            } else if !names.insert(f.name.as_str()) {
                bad.push(format!("«{}» is kept twice", f.name));
            }
            if f.language.trim().is_empty() || f.purpose.trim().is_empty() {
                bad.push(format!(
                    "«{}» needs its language and what it does on board",
                    f.name
                ));
            }
        }
        out.push(if bad.is_empty() {
            Check::pass("flight")
        } else {
            Check::fail("flight", bad.join("; "))
        });
    }
    out
}

/// Rust source with its comments removed and its string and character
/// literals emptied, so a check that reads code reads code.
///
/// The checks below once searched the raw text, so a hole that mentioned
/// `Sense::AtLeast` in a comment passed 7e while applying `Sense::AtMost`, and
/// a string saying ".sin()" failed the maths rule. Lifetimes (`'a`) are kept.
pub fn code_only(src: &str) -> String {
    let c: Vec<char> = src.chars().collect();
    let mut o = String::with_capacity(src.len());
    let mut i = 0;
    while i < c.len() {
        match c[i] {
            '/' if c.get(i + 1) == Some(&'/') => {
                while i < c.len() && c[i] != '\n' {
                    i += 1;
                }
            }
            '/' if c.get(i + 1) == Some(&'*') => {
                let mut depth = 1;
                i += 2;
                while i < c.len() && depth > 0 {
                    if c[i] == '/' && c.get(i + 1) == Some(&'*') {
                        depth += 1;
                        i += 2;
                    } else if c[i] == '*' && c.get(i + 1) == Some(&'/') {
                        depth -= 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                o.push(' ');
            }
            'r' if matches!(c.get(i + 1), Some('"' | '#'))
                && (i == 0 || !(c[i - 1].is_alphanumeric() || c[i - 1] == '_')) =>
            {
                // A raw string: r"..." or r#"..."#, with as many hashes.
                let mut j = i + 1;
                let mut hashes = 0;
                while c.get(j) == Some(&'#') {
                    hashes += 1;
                    j += 1;
                }
                if c.get(j) != Some(&'"') {
                    o.push(c[i]);
                    i += 1;
                    continue;
                }
                j += 1;
                loop {
                    if j >= c.len() {
                        break;
                    }
                    if c[j] == '"' && (0..hashes).all(|k| c.get(j + 1 + k) == Some(&'#')) {
                        j += 1 + hashes;
                        break;
                    }
                    j += 1;
                }
                o.push_str("\"\"");
                i = j;
            }
            '"' => {
                i += 1;
                while i < c.len() && c[i] != '"' {
                    if c[i] == '\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1;
                o.push_str("\"\"");
            }
            '\'' if c.get(i + 1) == Some(&'\\') || c.get(i + 2) == Some(&'\'') => {
                // A character literal, not a lifetime.
                i += 1;
                if c.get(i) == Some(&'\\') {
                    i += 1;
                }
                while i < c.len() && c[i] != '\'' {
                    i += 1;
                }
                i += 1;
                o.push_str("' '");
            }
            ch => {
                o.push(ch);
                i += 1;
            }
        }
    }
    o
}

/// The platform's transcendental and root functions, by method name.
const PLATFORM_MATHS: &[&str] = &[
    "sin", "cos", "tan", "asin", "acos", "atan", "atan2", "sin_cos", "sinh", "cosh", "tanh",
    "asinh", "acosh", "atanh", "exp", "exp2", "exp_m1", "ln", "ln_1p", "log", "log2", "log10",
    "powf", "powi", "sqrt", "cbrt", "hypot",
];

/// Every call into the platform's maths library in `src`: `x.sin()`,
/// `x . sin ()`, `f64::sin(x)`. Comments and strings are not code and are not
/// read. `pmath::sin(x)` is the portable route and is not a match.
pub fn platform_maths(src: &str) -> Vec<String> {
    let code: String = code_only(src)
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let mut found = Vec::new();
    for name in PLATFORM_MATHS {
        for (form, shown) in [
            (format!(".{name}("), format!(".{name}()")),
            (format!("f64::{name}("), format!("f64::{name}()")),
            (format!("f32::{name}("), format!("f32::{name}()")),
        ] {
            // The dot (or `::`) before the name and the `(` after it keep
            // `.sin(` from matching `.asin(` or `.sinh(`.
            if code.contains(&form) {
                found.push(shown);
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

/// The per-node checks.
/// A row answered by a table or by its children holds together: exactly one
/// behaviour, a table that can be read, children that are its group's.
/// Nothing for any other row.
fn behaviour_checks(sh: &Sheet, tree: &Tree) -> Vec<Check> {
    let mut out = Vec::new();
    let binding = |b: &str| sh.inputs.iter().find(|i| i.binding == b);
    if let Some(l) = &sh.lookup {
        let mut bad = Vec::new();
        if sh.is_declared() {
            bad.push("a stated row is its value, not a table".to_string());
        }
        if !sh.method.text.trim().is_empty() || !sh.steps.is_empty() || sh.children.is_some() {
            bad.push(
                "a row has one behaviour, and this one also has a method, steps or children".into(),
            );
        }
        if !sh.publishes.is_empty() {
            bad.push("a table answers one output, and this row publishes more".into());
        }
        if binding(&l.by).is_none() {
            bad.push(format!("by = '{}' names none of its inputs", l.by));
        }
        if l.x.len() != l.y.len() || l.x.len() < 2 {
            bad.push(format!(
                "x and y must be the same length, two rows at least — they are {} and {}",
                l.x.len(),
                l.y.len()
            ));
        }
        if l.x.iter().chain(&l.y).any(|v| !v.is_finite()) {
            bad.push("every entry must be a finite number".into());
        }
        if l.x.windows(2).any(|w| w[1] <= w[0]) {
            bad.push("x must rise from row to row, each strictly above the last".into());
        }
        match l.read.as_str() {
            "linear" => {}
            "log" if l.y.iter().all(|v| *v > 0.0) => {}
            "log" => bad.push("read = 'log' needs every y above zero".into()),
            r => bad.push(format!("read = '{r}' is neither 'linear' nor 'log'")),
        }
        out.push(if bad.is_empty() {
            Check::pass("lookup")
        } else {
            Check::fail("lookup", bad.join("; "))
        });
    }
    if let Some(c) = &sh.children {
        let mut bad = Vec::new();
        if sh.is_declared() {
            bad.push("a stated row is its value, not its children's".to_string());
        }
        // The group and every group under it: where its children are.
        let mut under = vec![c.group.clone()];
        let mut i = 0;
        while i < under.len() {
            for (id, g) in &tree.groups {
                if g.parent == under[i] && !under.contains(id) {
                    under.push(id.clone());
                }
            }
            i += 1;
        }
        if !tree.groups.contains_key(&c.group) {
            bad.push(format!("group = '{}' is not a group", c.group));
        } else if under.contains(&sh.parent) {
            bad.push(format!(
                "the row is inside {}: a block's answer is read from its children, not from itself",
                c.group
            ));
        }
        let outputs = 1 + sh.publishes.len();
        if c.from.len() != outputs {
            bad.push(format!(
                "from names {} port(s) for {outputs} output(s): one each, primary first",
                c.from.len()
            ));
        }
        for b in &c.from {
            match binding(b) {
                None => bad.push(format!("from names '{b}', none of its inputs")),
                Some(i) => {
                    let producer = i.var.split('.').next().unwrap_or("");
                    let inside = tree
                        .sheets
                        .get(producer)
                        .is_some_and(|p| under.contains(&p.parent));
                    if !inside {
                        bad.push(format!(
                            "'{b}' reads {}, which is not a port of a child in {}",
                            i.var, c.group
                        ));
                    }
                }
            }
        }
        out.push(if bad.is_empty() {
            Check::pass("children")
        } else {
            Check::fail("children", bad.join("; "))
        });
    }
    out
}

/// What an output says of its value holds together: words the schema knows,
/// a state its row can have, an open value with its owner, its gate and a
/// range, a parameter only on a stated value. Nothing for a row whose outputs
/// say none of it.
fn port_checks(sh: &Sheet) -> Vec<Check> {
    let mut ports = vec![("output".to_string(), &sh.port, sh.lower, sh.upper)];
    for pb in &sh.publishes {
        ports.push((format!("publishes {}", pb.id), &pb.port, pb.lower, pb.upper));
    }
    if ports.iter().all(|(_, p, ..)| **p == Default::default()) {
        return Vec::new();
    }
    let mut bad = Vec::new();
    for (place, p, lower, upper) in ports {
        let one_of = |what: &str, v: &str, among: &[&str], bad: &mut Vec<String>| {
            if !v.is_empty() && !among.contains(&v) {
                bad.push(format!(
                    "{place}: {what} = '{v}' is not one of {}",
                    among.join(", ")
                ));
            }
        };
        one_of(
            "state",
            &p.state,
            &["decided", "allocated", "open", "achieved"],
            &mut bad,
        );
        one_of(
            "maturity",
            &p.maturity,
            &["estimated", "calculated", "measured"],
            &mut bad,
        );
        one_of(
            "parameter",
            &p.parameter,
            &["programme", "system", "subsystem"],
            &mut bad,
        );
        let stated = sh.is_declared() || sh.kind == "required";
        match p.state.as_str() {
            "decided" if !sh.is_declared() => bad.push(format!(
                "{place}: decided is a value stated at this level, and this row is {}",
                sh.kind
            )),
            "allocated" if sh.kind != "required" => bad.push(format!(
                "{place}: allocated is a bound handed to a child, and this row is {}",
                sh.kind
            )),
            "achieved" if stated => bad.push(format!(
                "{place}: achieved is computed, and this row states its value"
            )),
            "open" if !stated => bad.push(format!(
                "{place}: open is a value still to decide, and this row computes its own"
            )),
            "open" => {
                if p.open_owner.is_empty() || p.open_due.is_empty() {
                    bad.push(format!(
                        "{place}: an open value names who owns it (open_owner) and the gate it is due by (open_due)"
                    ));
                }
                if !(lower.is_finite() && upper.is_finite() && lower < upper) {
                    bad.push(format!(
                        "{place}: an open value carries the range it may still take — a finite lower and upper"
                    ));
                }
            }
            _ => {}
        }
        if !p.parameter.is_empty() && !sh.is_declared() {
            bad.push(format!(
                "{place}: a parameter is a stated value, and this row is {}",
                sh.kind
            ));
        }
    }
    vec![if bad.is_empty() {
        Check::pass("port")
    } else {
        Check::fail("port", bad.join("; "))
    }]
}

pub fn gate_node(sh: &Sheet, tree: &Tree) -> Vec<Check> {
    let mut out = Vec::new();
    let holes = read_holes(&sh.dir);

    // A seeded row is checked for the four things a seed is responsible for and
    // nothing else. Everything a person has yet to write is the gap pass's to
    // report, and a gate that refuses most of a tree for six months is a gate
    // nobody reads.
    if sh.is_seeded() {
        let mut missing = Vec::new();
        for (name, v) in [
            ("id", &sh.id),
            ("label", &sh.label),
            ("parent", &sh.parent),
            ("owner", &sh.owner),
        ] {
            if v.trim().is_empty() {
                missing.push(name);
            }
        }
        out.push(if missing.is_empty() {
            Check::pass("seeded")
        } else {
            Check::fail(
                "seeded",
                format!("a seeded row still needs: {}", missing.join(", ")),
            )
        });
        out.push(if tree.groups.contains_key(&sh.parent) {
            Check::pass("parent")
        } else {
            Check::fail("parent", format!("'{}' is not a group", sh.parent))
        });
        out.push(versions_check(sh));
        out.extend(method_checks(sh));
        out.extend(lesson_check(sh, tree));
        out.extend(behaviour_checks(sh, tree));
        out.extend(port_checks(sh));
        let gaps = emit::gap_pass(sh, &holes);
        out.push(if gaps.is_empty() {
            Check::pass("gap-pass")
        } else {
            Check::note("gap-pass", gaps.join("; "))
        });
        return out;
    }
    out.push(versions_check(sh));
    out.extend(method_checks(sh));
    out.extend(lesson_check(sh, tree));
    out.extend(behaviour_checks(sh, tree));
    out.extend(port_checks(sh));

    // 1 — the sheet validates; no required field is blank.
    let mut missing = Vec::new();
    for (name, v) in [
        ("id", &sh.id),
        ("label", &sh.label),
        ("owner", &sh.owner),
        ("question", &sh.question),
        ("expression", &sh.expression),
        ("source", &sh.source),
        ("type", &sh.ty),
        ("unit", &sh.unit),
        ("symbol", &sh.symbol),
        ("reason_lower", &sh.reason_lower),
        ("reason_upper", &sh.reason_upper),
    ] {
        if v.trim().is_empty() {
            missing.push(name);
        }
    }
    out.push(if missing.is_empty() {
        Check::pass("schema")
    } else {
        Check::fail(
            "schema",
            format!("required fields blank: {}", missing.join(", ")),
        )
    });

    // 2 — a computed node declares at least one input.
    out.push(if sh.is_declared() || !sh.inputs.is_empty() {
        Check::pass("inputs")
    } else {
        Check::fail(
            "inputs",
            "a computed node with no declared input claims to compute something from nothing"
                .into(),
        )
    });

    // 3 — a declared value names a source and a confirmation.
    out.push(
        if !sh.is_declared() || (sh.value.is_some() && !sh.confirmed_by.trim().is_empty()) {
            Check::pass("declared-value")
        } else {
            Check::fail(
                "declared-value",
                "a declared value needs a number, a source and who confirmed it".into(),
            )
        },
    );

    // 3b — a row that publishes a set declares each member as fully as it
    // declares its own answer.
    //
    // The exception to one row, one answer exists for one shape of thing and it
    // is not a licence to publish a bag of numbers. A member with no bounds is a
    // member with no guard, and a member with no reason for its bounds is a
    // guard the next person deletes; a member with no symbol has no field to
    // assign in a hole body and no name on a page.
    let mut pub_bad = Vec::new();
    if sh.is_declared() && !sh.publishes.is_empty() {
        pub_bad.push(
            "a declared row states one measured number; a set is computed from the rows that \
             measured its members"
                .to_string(),
        );
    }
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for pb in &sh.publishes {
        if pb.id.trim().is_empty() {
            pub_bad.push("a publishes block with no id".into());
            continue;
        }
        if pb.id.contains('.') {
            pub_bad.push(format!(
                "'{}' contains a dot, and a dot is what separates a node from its member",
                pb.id
            ));
        }
        if !seen.insert(pb.id.as_str()) {
            pub_bad.push(format!("'{}' is published twice", pb.id));
        }
        for (what, v) in [
            ("symbol", &pb.symbol),
            ("label", &pb.label),
            ("type", &pb.ty),
            ("unit", &pb.unit),
            ("reason_lower", &pb.reason_lower),
            ("reason_upper", &pb.reason_upper),
        ] {
            if v.trim().is_empty() {
                pub_bad.push(format!("'{}' has no {}", pb.id, what));
            }
        }
        if pb.lower >= pb.upper {
            pub_bad.push(format!(
                "'{}' declares a lower bound of {} at or above its upper bound of {}",
                pb.id, pb.lower, pb.upper
            ));
        }
    }
    // A fixture on a set row has to say which member it is about, and the name
    // has to be one of them. Defaulting silently to the primary would make a
    // typo into a test that passes against the wrong variable.
    for f in &sh.fixtures {
        if f.variable.is_empty() {
            if !sh.publishes.is_empty() {
                pub_bad.push(format!(
                    "the fixture '{}' names no variable, and this row publishes {} of them",
                    f.label,
                    sh.publishes.len() + 1
                ));
            }
        } else if f.variable != sh.symbol && !sh.publishes.iter().any(|pb| pb.id == f.variable) {
            pub_bad.push(format!(
                "the fixture '{}' names '{}', which this row does not publish",
                f.label, f.variable
            ));
        }
    }
    out.push(if pub_bad.is_empty() {
        Check::pass("publishes")
    } else {
        Check::fail("publishes", pub_bad.join("; "))
    });

    // 4 — every input resolves, and the declared type agrees with the producer.
    //
    // An input may name a node — the ordinary case — or one member of a set a
    // node publishes, as `<node id>.<publish id>`. A node id never contains a
    // dot, so the two cannot be confused, and the member is type-checked against
    // the publish block rather than against the producing row's own answer.
    let mut bad = Vec::new();
    for i in &sh.inputs {
        if let Some((node, member)) = i.var.split_once('.') {
            match tree.sheets.get(node) {
                None => bad.push(format!("'{}' names no node", node)),
                Some(p) => match p.publishes.iter().find(|pb| pb.id == member) {
                    None => bad.push(format!(
                        "'{}' names no variable {} publishes — it publishes {}",
                        i.var,
                        p.id,
                        if p.publishes.is_empty() {
                            "only its own answer".to_string()
                        } else {
                            p.publishes
                                .iter()
                                .map(|pb| pb.id.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    )),
                    Some(pb) if pb.ty != i.ty => bad.push(format!(
                        "'{}' expects {} but {}.{} publishes {}",
                        i.binding, i.ty, p.id, pb.id, pb.ty
                    )),
                    Some(_) if p.state == "deprecated" => bad.push(format!(
                        "'{}' is deprecated and may not be a new dependency",
                        p.id
                    )),
                    Some(_) => {}
                },
            }
            continue;
        }
        match tree.sheets.get(&i.var) {
            None => bad.push(format!("'{}' names no node", i.var)),
            Some(p) if p.ty != i.ty => bad.push(format!(
                "'{}' expects {} but {} publishes {}",
                i.binding, i.ty, p.id, p.ty
            )),
            Some(p) if p.state == "deprecated" => bad.push(format!(
                "'{}' is deprecated and may not be a new dependency",
                p.id
            )),
            _ => {}
        }
    }
    out.push(if bad.is_empty() {
        Check::pass("contract")
    } else {
        Check::fail("contract", bad.join("; "))
    });

    // 5 — every source resolves to an entry in sources/.
    let mut unresolved = BTreeSet::new();
    if !tree.sources.contains_key(&sh.source) {
        unresolved.insert(sh.source.clone());
    }
    for f in &sh.fixtures {
        if !tree.sources.contains_key(&f.source) {
            unresolved.insert(f.source.clone());
        }
    }
    out.push(if unresolved.is_empty() {
        Check::pass("sources")
    } else {
        Check::fail(
            "sources",
            format!(
                "these cite nothing in sources/: {}",
                unresolved.into_iter().collect::<Vec<_>>().join(", ")
            ),
        )
    });

    // 6 — regenerate and compare. A hand edit outside a hole fails here, which
    //     is what makes the generated region genuinely owned by the generator.
    let mut drift = Vec::new();
    for (name, want) in [
        ("model.rs", emit::model_rs(sh, &holes)),
        ("contract.rs", emit::contract_rs(sh)),
        ("mod.rs", emit::mod_rs(sh)),
        ("evidence.rs", emit::evidence_rs(sh)),
    ] {
        let p = sh.dir.join(name);
        match std::fs::read_to_string(&p) {
            // The committed file is formatted; what the generator emits is not
            // yet. Comparing after normalising whitespace asks the question the
            // check is actually for — did the *content* drift — rather than
            // whether the formatter has run.
            Ok(got) if got == formatted(&want) || normalise(&got) == normalise(&want) => {}
            Ok(_) => drift.push(name),
            Err(_) => drift.push(name),
        }
    }
    out.push(if drift.is_empty() {
        Check::pass("regenerate")
    } else {
        Check::fail(
            "regenerate",
            format!(
                "{} differ from what the sheet generates — run `cargo xtask docs`",
                drift.join(", ")
            ),
        )
    });

    // 7 — the gap pass.
    let gaps = emit::gap_pass(sh, &holes);
    out.push(if gaps.is_empty() {
        Check::pass("gap-pass")
    } else {
        Check::note("gap-pass", gaps.join("; "))
    });

    // 7b — criticality, and what it buys. It decides how many people read the
    //      node and whether the hole is filled twice by different model
    //      families, so a word nobody recognises would silently choose the
    //      cheaper answer.
    let crit = sh.criticality.as_str();
    out.push(if crit == "minor" || crit == "significant" {
        Check::pass("criticality")
    } else {
        Check::fail(
            "criticality",
            format!("'{crit}' is neither 'minor' nor 'significant' — it decides reviewer count and whether differential fill runs"),
        )
    });

    // 7c — the prior implementation, when there is one. Its numbers may never
    //      be fixtures: an implementation cannot supply its own expected
    //      values. They belong in parity.csv, where a disagreement is a finding
    //      about one of the two rather than a check either has passed.
    let migrated = !sh.migrated_from.trim().is_empty();
    out.push(if !migrated || sh.dir.join("parity.csv").is_file() {
        Check::pass("parity")
    } else {
        Check::note(
            "parity",
            format!(
                "migrated_from names '{}' and there is no parity.csv beside it — the old \
                 implementation is a second opinion only once its numbers are recorded",
                sh.migrated_from
            ),
        )
    });

    // 7d — a requirement says which way it binds.
    //
    //      A bound is meaningless until it states which side of it is safe.
    //      "The design sustains Ap 200" and "the design needs Ap 200" are the
    //      same number and opposite requirements: read the wrong way, a closure
    //      reports a comfortable margin for a spacecraft that is about to be
    //      destroyed.
    //
    //      Which rows this reaches is taken from the graph rather than from a
    //      naming convention: a row is a requirement if it is declared one, or
    //      if some closure reads it as its `req` binding. A convention can be
    //      dodged by renaming a folder; a contract edge cannot, and the edge is
    //      the thing that actually makes the comparison happen.
    let bound_as_requirement = tree.sheets.values().any(|o| {
        o.inputs
            .iter()
            .any(|i| i.var == sh.id && i.binding == "req")
    });
    let is_requirement = sh.kind == "required" || bound_as_requirement;
    let stated = matches!(sh.sense.trim(), "<=" | ">=");
    out.push(if !is_requirement || stated {
        Check::pass("sense")
    } else if sh.sense.trim().is_empty() {
        Check::fail(
            "sense",
            "a requirement with no declared sense — say `sense = \"<=\"` if the achieved value \
             must stay under this bound, or `sense = \">=\"` if it must reach it. Defaulting \
             either is how a silently wrong bound gets shipped"
                .to_string(),
        )
    } else {
        Check::fail(
            "sense",
            format!(
                "sense is '{}' — it must be exactly \"<=\" or \">=\"",
                sh.sense.trim()
            ),
        )
    });

    // 7e — the sense the sheet declares is the sense the code applies.
    //
    //      The closure's hole says `Sense::AtLeast` or `Sense::AtMost`, and that
    //      is what actually runs. The requirement row now declares the same
    //      thing. Two statements of one fact drift, and this one drifts
    //      silently: the margin still computes, still has a plausible sign, and
    //      is wrong in the direction nobody looks.
    //
    //      Checked here rather than trusted because the whole point of 7d is
    //      that a bound read the wrong way is invisible.
    let mut disagree = Vec::new();
    for i in &sh.inputs {
        if i.binding != "req" {
            continue;
        }
        let Some(req) = tree.sheets.get(producer_of(&i.var)) else {
            continue;
        };
        let want = match req.sense.trim() {
            "<=" => "Sense::AtMost",
            ">=" => "Sense::AtLeast",
            _ => continue,
        };
        let other = if want == "Sense::AtMost" {
            "Sense::AtLeast"
        } else {
            "Sense::AtMost"
        };
        let body: String = code_only(&holes.values().cloned().collect::<Vec<_>>().join("\n"));
        // A method says it in its own words: `margin_at_most` for `<=`,
        // `margin_at_least` for `>=`, read from its lines and not its comments.
        let method: String = sh
            .method
            .text
            .lines()
            .map(|l| l.split('#').next().unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n");
        let says = |sense: &str| {
            body.contains(sense)
                || method.contains(if sense == "Sense::AtMost" {
                    "margin_at_most"
                } else {
                    "margin_at_least"
                })
        };
        if says(other) && !says(want) {
            disagree.push(format!(
                "{} declares sense {:?} so this must apply {want}, and it applies {other}",
                req.id,
                req.sense.trim()
            ));
        }
    }
    out.push(if disagree.is_empty() {
        Check::pass("sense-applied")
    } else {
        Check::fail("sense-applied", disagree.join(", "))
    });

    // 7f — a row that returns its one input unchanged is not a row.
    //
    // Five achieved rows in the solar subsystem had bodies that read, in full,
    // `let ach = conclusion;`. Each produced the same number as the row beneath
    // it under every perturbation the tree allows -- it WAS that row, under a
    // second name. A reader met the same answer twice with nothing to tell them
    // which to believe, and the closure those rows were meant to be half of did
    // not exist, because nothing read the requirement.
    //
    // Three exemptions, each for a relay that is doing real work.
    //
    // A CROSSING relays without computing because that is precisely its job,
    // and §20.3 says so.
    //
    // A relay ACROSS A LAYER is the receiving layer naming the thing in its own
    // vocabulary: sys_space_environment_kp is layer 2's word for a member of
    // the solar driver set, and without it every layer-2 consumer would reach
    // into a subsystem by its internal member names -- which is the reaching-in
    // the crossing rule exists to forbid. Same number, different vocabulary,
    // and the vocabulary is the point.
    //
    // A RETIRED row is not asked to justify itself. It is on its way out, and
    // being a relay is usually why.
    //
    // Everything else earns its place by producing a number the tree does not
    // already hold.
    let identity = {
        let body: String = holes.values().cloned().collect::<Vec<_>>().join("\n");
        let mut found = None;
        for line in body.lines() {
            let t = line.trim().trim_end_matches(';');
            if t.starts_with("//") || !t.starts_with("let ") {
                continue;
            }
            // `let <name>: <Type> = <rhs>` with nothing done to the right-hand
            // side, where the right-hand side is one of this row's own inputs.
            let Some((_, rhs)) = t.split_once('=') else {
                continue;
            };
            let rhs = rhs.trim();
            if sh.inputs.iter().any(|i| i.binding == rhs) && sh.inputs.len() == 1 {
                found = Some(rhs.to_string());
                break;
            }
        }
        found
    };
    let same_layer = |binding: &str| {
        sh.inputs
            .iter()
            .find(|i| i.binding == binding)
            .and_then(|i| tree.sheets.get(producer_of(&i.var)))
            .is_some_and(|p| p.layer == sh.layer)
    };
    out.push(match identity {
        Some(ref binding)
            if sh.crosses_to.trim().is_empty()
                && sh.state != "deprecated"
                && same_layer(binding) =>
        {
            Check::fail(
                "no-identity",
                format!(
                "this row's answer is `{binding}` unchanged, and `{binding}` is its only input — \
                 so it publishes a number the tree already has under another name. Give it \
                 something to compute, or let its consumers read the producer directly. Only a \
                 crossing may relay without computing"
                ),
            )
        }
        _ => Check::pass("no-identity"),
    });

    // 8 — fixture provenance. The one rule the evidence model rests on.
    let mut badfx = Vec::new();
    for f in &sh.fixtures {
        if !matches!(
            f.provenance.as_str(),
            "independent-derivation" | "published-source" | "independent-tool" | "physical-bound"
        ) {
            badfx.push(format!("'{}' has provenance '{}'", f.label, f.provenance));
        }
        if f.tolerance <= 0.0 {
            badfx.push(format!("'{}' has a non-positive tolerance", f.label));
        }
    }
    out.push(if badfx.is_empty() {
        Check::pass("provenance")
    } else {
        Check::fail(
            "provenance",
            format!(
                "an expected value may not come from the code under test: {}",
                badfx.join("; ")
            ),
        )
    });

    // 9 — the declared limits are ordered and the declared value sits inside them.
    let mut dom = Vec::new();
    if sh.lower >= sh.upper {
        dom.push("the lower limit is not below the upper".to_string());
    }
    if let Some(v) = sh.value {
        if v < sh.lower || v > sh.upper {
            dom.push(format!(
                "the declared value {v} is outside its own declared range"
            ));
        }
    }
    out.push(if dom.is_empty() {
        Check::pass("domain")
    } else {
        Check::fail("domain", dom.join("; "))
    });

    // 10 — the portable maths rule. The kernel crates may not reach the
    //      platform maths library, or cross-face agreement fails on night one
    //      for a reason that is not a defect.
    let mut leaks = Vec::new();
    for (n, body) in &holes {
        for bad in platform_maths(body) {
            leaks.push(format!("hole {n} calls {bad} — route it through pmath"));
        }
    }
    out.push(if leaks.is_empty() {
        Check::pass("portable-maths")
    } else {
        Check::fail("portable-maths", leaks.join("; "))
    });

    out
}

/// The assembly validations. These can only be asked where the whole tree is
/// visible, which is why they belong here and not in a per-node gate.
pub fn validate_tree(tree: &Tree) -> Vec<Check> {
    let mut out = Vec::new();

    // V17 — the risk register is well formed: every risk registered once, on
    //       a row of the register, with a level, an owner and what it would
    //       cost.
    // V18 — every risk a version moves is a registered one. A risk that
    //       exists only in a version's record is a risk nobody owns.
    let reg = crate::derisk::register_problems(tree);
    let (moves, rest): (Vec<String>, Vec<String>) = reg
        .into_iter()
        .partition(|m| m.contains("no risk-register row registers"));
    out.push(if rest.is_empty() {
        Check::pass("V17 the risk register is well formed")
    } else {
        Check::fail("V17 the risk register is well formed", rest.join("; "))
    });
    out.push(if moves.is_empty() {
        Check::pass("V18 every risk moved is registered")
    } else {
        Check::fail("V18 every risk moved is registered", moves.join("; "))
    });

    // V1 — every edge endpoint names a row that exists.
    let mut dangling = Vec::new();
    for sh in tree.ordered() {
        for i in &sh.inputs {
            if !var_resolves(tree, &i.var) {
                dangling.push(format!("{} -> {}", sh.id, i.var));
            }
        }
        for k in &sh.kpis {
            if !tree.sheets.contains_key(k) {
                dangling.push(format!("{} contributes to {}", sh.id, k));
            }
        }
    }
    out.push(if dangling.is_empty() {
        Check::pass("V1 edge endpoints exist")
    } else {
        Check::fail("V1 edge endpoints exist", dangling.join(", "))
    });

    // V2 — no edge is declared twice. Each edge is declared exactly once, by
    //      the end the edge changes.
    let mut dup = Vec::new();
    for sh in tree.ordered() {
        let mut seen = BTreeSet::new();
        for i in &sh.inputs {
            if !seen.insert(i.var.clone()) {
                dup.push(format!("{} reads {} twice", sh.id, i.var));
            }
        }
    }
    out.push(if dup.is_empty() {
        Check::pass("V2 no duplicate edges")
    } else {
        Check::fail("V2 no duplicate edges", dup.join(", "))
    });

    // V3 — every node's parent resolves to a layer group.
    let mut orphans = Vec::new();
    for sh in tree.ordered() {
        if !tree.groups.contains_key(&sh.parent) {
            orphans.push(format!(
                "{} hangs under '{}', which is not a group",
                sh.id, sh.parent
            ));
        }
    }
    out.push(if orphans.is_empty() {
        Check::pass("V3 parents resolve")
    } else {
        Check::fail("V3 parents resolve", orphans.join(", "))
    });

    // V4 — every cycle in the derivation graph is declared in a case. A cycle
    //      is a cross-branch property: two individually clean branches can form
    //      one, so this belongs to the merged state and never to a per-node gate.
    let declared: BTreeSet<String> = tree
        .cases
        .values()
        .flat_map(|c| c.cycles.iter())
        .flat_map(|cy| cy.nodes.iter().cloned())
        .collect();
    let cycles = find_cycles(tree, &declared);
    out.push(if cycles.is_empty() {
        Check::pass("V4 every cycle is declared")
    } else {
        Check::fail(
            "V4 every cycle is declared",
            format!("undeclared loop: {}", cycles.join(" -> ")),
        )
    });

    // V5 — every computed node declares at least one input.
    let bad: Vec<String> = tree
        .ordered()
        .iter()
        .filter(|s| !s.is_declared() && !s.is_seeded() && s.inputs.is_empty())
        .map(|s| s.id.clone())
        .collect();
    out.push(if bad.is_empty() {
        Check::pass("V5 computed nodes have inputs")
    } else {
        Check::fail("V5 computed nodes have inputs", bad.join(", "))
    });

    // V6 — no two siblings resolve to the same folder name.
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut collide = Vec::new();
    for sh in tree.ordered() {
        if !seen.insert((sh.crate_name.clone(), sh.folder.clone())) {
            collide.push(format!("{}/{}", sh.crate_name, sh.folder));
        }
    }
    out.push(if collide.is_empty() {
        Check::pass("V6 no folder collisions")
    } else {
        Check::fail("V6 no folder collisions", collide.join(", "))
    });

    // V7 — every layer group has an owner. An ownerless subsystem routes
    //      reviews nowhere, silently.
    let un: Vec<String> = tree
        .groups
        .values()
        .filter(|g| g.owner.trim().is_empty())
        .map(|g| g.id.clone())
        .collect();
    out.push(if un.is_empty() {
        Check::pass("V7 every layer has an owner")
    } else {
        Check::fail("V7 every layer has an owner", un.join(", "))
    });

    // V8 — every source reference resolves, and nothing depends on a
    //      superseded one without being listed.
    let mut unresolved = BTreeSet::new();
    let mut superseded = BTreeSet::new();
    for sh in tree.ordered() {
        if sh.is_seeded() {
            continue;
        }
        for s in std::iter::once(&sh.source).chain(sh.fixtures.iter().map(|f| &f.source)) {
            match tree.sources.get(s) {
                None => {
                    unresolved.insert(format!("{} cites {}", sh.id, s));
                }
                Some(src) if src.status != "current" => {
                    superseded.insert(format!("{} cites {} ({})", sh.id, s, src.status));
                }
                _ => {}
            }
        }
    }
    out.push(if unresolved.is_empty() {
        Check::pass("V8 sources resolve")
    } else {
        Check::fail(
            "V8 sources resolve",
            unresolved.into_iter().collect::<Vec<_>>().join(", "),
        )
    });
    out.push(if superseded.is_empty() {
        Check::pass("V9 no superseded sources")
    } else {
        Check::fail(
            "V9 no superseded sources",
            superseded.into_iter().collect::<Vec<_>>().join(", "),
        )
    });

    // V10 — every relation edge names groups that exist.
    let mut badrel = Vec::new();
    for r in &tree.relations {
        if !tree.groups.contains_key(&r.from) || !tree.groups.contains_key(&r.to) {
            badrel.push(format!("{} -> {}", r.from, r.to));
        }
        if r.why.trim().is_empty() {
            badrel.push(format!(
                "{} -> {} is unlabelled, which is itself a finding",
                r.from, r.to
            ));
        }
    }
    out.push(if badrel.is_empty() {
        Check::pass("V10 relations resolve and are labelled")
    } else {
        Check::fail("V10 relations resolve and are labelled", badrel.join(", "))
    });

    // V11b — a loop belongs to the smallest block that holds it. One declared
    //        on a block is held to that; one declared on none is noted with
    //        the block it belongs on, for its owner to move it there.
    let chain = |g: &str| -> Vec<String> {
        let mut out = vec![g.to_string()];
        let mut at = g.to_string();
        while let Some(p) = tree.groups.get(&at).map(|x| x.parent.clone()) {
            if p.is_empty() || out.contains(&p) {
                break;
            }
            out.push(p.clone());
            at = p;
        }
        out
    };
    let (mut misplaced, mut unplaced) = (Vec::new(), Vec::new());
    for cy in &tree.cycles {
        let chains: Vec<Vec<String>> = cy
            .nodes
            .iter()
            .filter_map(|n| tree.sheets.get(n))
            .map(|sh| chain(&sh.parent))
            .collect();
        let holds = chains.first().and_then(|first| {
            first
                .iter()
                .find(|g| chains.iter().all(|c| c.contains(g)))
                .cloned()
        });
        let Some(holds) = holds else { continue };
        let named = cy.nodes.first().cloned().unwrap_or_default();
        if cy.on.is_empty() {
            unplaced.push(format!(
                "the loop through {named} and {} more is declared on no block; the smallest that holds it is {holds}",
                cy.nodes.len().saturating_sub(1)
            ));
        } else if cy.on != holds {
            misplaced.push(format!(
                "the loop through {named} is declared on {}, and the smallest block that holds it is {holds}",
                cy.on
            ));
        }
    }
    out.push(if !misplaced.is_empty() {
        Check::fail(
            "V11b loops on the block that holds them",
            misplaced.join("; "),
        )
    } else if !unplaced.is_empty() {
        Check::note(
            "V11b loops on the block that holds them",
            unplaced.join("; "),
        )
    } else {
        Check::pass("V11b loops on the block that holds them")
    });

    // V11 — every declared cycle names nodes that exist and a convergence
    //       variable inside the loop.
    let mut badcy = Vec::new();
    for c in tree.cases.values() {
        for cy in &c.cycles {
            for n in &cy.nodes {
                if !tree.sheets.contains_key(n) {
                    badcy.push(format!(
                        "case {} declares a cycle through {}, which does not exist",
                        c.id, n
                    ));
                }
            }
            if !cy.nodes.contains(&cy.converge_on) {
                badcy.push(format!(
                    "case {} converges on {}, which is not in its own cycle",
                    c.id, cy.converge_on
                ));
            }
            if cy.tolerance <= 0.0 || cy.max_iter == 0 {
                badcy.push(format!(
                    "case {} declares a cycle with no usable stopping rule",
                    c.id
                ));
            }
        }
    }
    out.push(if badcy.is_empty() {
        Check::pass("V11 declared cycles are well formed")
    } else {
        Check::fail("V11 declared cycles are well formed", badcy.join(", "))
    });

    // V12 — one crate per owner.
    //
    // Every row of a group lives in the same crate, and no crate holds rows
    // from two layers. This is the isolation rule checked rather than trusted:
    // a group whose rows are in two crates is a group that two teams have to
    // edit together, and it happens silently — a routing rule that keys on the
    // wrong field, a row added under the old convention — until somebody
    // notices a change to one layer touching three crates.
    let mut byg: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut byc: BTreeMap<&str, BTreeSet<u8>> = BTreeMap::new();
    for sh in tree.sheets.values() {
        byg.entry(sh.parent.as_str())
            .or_default()
            .insert(sh.crate_name.as_str());
        byc.entry(sh.crate_name.as_str())
            .or_default()
            .insert(sh.layer);
    }
    let mut spread: Vec<String> = byg
        .iter()
        .filter(|(_, c)| c.len() > 1)
        .map(|(g, c)| {
            format!(
                "{} is split across {}",
                g,
                c.iter().copied().collect::<Vec<_>>().join(" and ")
            )
        })
        .collect();
    spread.extend(byc.iter().filter(|(_, l)| l.len() > 1).map(|(c, l)| {
        format!(
            "{} holds layers {}",
            c,
            l.iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" and ")
        )
    }));
    out.push(if spread.is_empty() {
        Check::pass("V12 one crate per owner")
    } else {
        Check::fail("V12 one crate per owner", spread.join(", "))
    });

    // V14 — no two rows claim the same place on the tree.
    //
    // `order` is what the display list sorts by, so two rows sharing one puts
    // them in an arbitrary order that depends on the map's iteration — stable
    // within a run, and free to swap when a row is added anywhere. It reads as
    // a reordering nobody made.
    //
    // Added because a subsystem added after the seed collided with an existing
    // interface row and nothing said so: the gate was green, the assembly was
    // green, and two rows sat at 720.
    let mut at: BTreeMap<u32, Vec<&str>> = BTreeMap::new();
    for sh in tree.ordered() {
        at.entry(sh.order).or_default().push(sh.id.as_str());
    }
    let clashes: Vec<String> = at
        .iter()
        .filter(|(_, ids)| ids.len() > 1)
        .map(|(o, ids)| format!("{o}: {}", ids.join(" and ")))
        .collect();
    out.push(if clashes.is_empty() {
        Check::pass("V14 one row per place")
    } else {
        Check::fail("V14 one row per place", clashes.join(", "))
    });

    // V15 — no two rows in one layer group carry the same label.
    //
    // The tree draws the label and nothing else. Two rows sharing one are two
    // identical lines a reader has to click to tell apart, and the required and
    // achieved halves of a closure are exactly the pair most likely to collide:
    // they ask about the same quantity on purpose. Five such pairs sat in the
    // solar layer — "Ap, sustained" twice, "F10.7, single day" twice — and the
    // only way to know which was the requirement was to open both.
    //
    // Scoped to the group rather than the whole tree, because the same short
    // label under two different subsystems is read with its parent and is not
    // ambiguous; within one list it is.
    let mut lbl: BTreeMap<(&str, &str), Vec<&str>> = BTreeMap::new();
    for sh in tree.ordered() {
        lbl.entry((sh.parent.as_str(), sh.label.as_str()))
            .or_default()
            .push(sh.id.as_str());
    }
    let twins: Vec<String> = lbl
        .iter()
        .filter(|(_, ids)| ids.len() > 1)
        .map(|((p, l), ids)| format!("{p} has \"{l}\" on {}", ids.join(" and ")))
        .collect();
    out.push(if twins.is_empty() {
        Check::pass("V15 one label per row in a group")
    } else {
        Check::fail("V15 one label per row in a group", twins.join(", "))
    });

    // V16 — the case supplies only what a run can take, and its groups name
    //       only real inputs.
    //
    // A value in `cases/` aimed at a row that is not declared is overwritten
    // the moment that row is evaluated, and the run reports the design's number
    // as though the file had been applied. That happened: two stored skies set
    // three rows that later became computed, and from then on "solar maximum"
    // returned the plain design under a storm's name. A key naming no row at
    // all was worse — the generator dropped it before the engine saw it.
    //
    // The Condition list is held to the same standard. It decides which half
    // of the Inputs page and of every uploaded CSV an input sits in, and a
    // name that is not a declared, published input would be a heading over
    // nothing — or, after a rename, an input quietly moved to Customer.
    let mut badcase = Vec::new();
    for c in tree.cases.values() {
        let mut check = |what: &str, k: &str| match tree.sheets.get(k) {
            None => badcase.push(format!("{} {what} {}, which is not a row", c.id, k)),
            Some(sh) if !sh.is_declared() => badcase.push(format!(
                "{} {what} {}, which is {} — a run overwrites it, so it would change \
                 nothing. Only a declared input can be set",
                c.id, k, sh.kind
            )),
            Some(sh) if sh.state != "published" => badcase.push(format!(
                "{} {what} {}, which is {} rather than published",
                c.id,
                k,
                if sh.is_seeded() {
                    "seeded"
                } else {
                    sh.state.as_str()
                }
            )),
            Some(_) => {}
        };
        for (k, _) in &c.supply {
            check("supplies", k);
        }
        for k in &c.conditions {
            check("lists as a condition", k);
        }
        let mut seen = BTreeSet::new();
        for k in &c.conditions {
            if !seen.insert(k.as_str()) {
                badcase.push(format!("{} lists {} as a condition twice", c.id, k));
            }
        }
    }
    if tree.cases.is_empty() {
        badcase.push("cases/ holds no case, and every face runs one by default".into());
    }
    out.push(if badcase.is_empty() {
        Check::pass("V16 the case supplies only what a run can take")
    } else {
        Check::fail(
            "V16 the case supplies only what a run can take",
            badcase.join(", "),
        )
    });

    // V13 — the browser face offers only rows it can actually answer.
    //
    // The demonstration subset is a hand-written list in a crate outside the
    // workspace, so `cargo test` never sees it and it drifts silently. Every
    // way it can drift ends the same way: a visitor clicks a node the page
    // offered and gets an error instead of a number, on the one face chosen
    // for people who have not installed anything.
    //
    // Read as text rather than linked, because linking it would put a
    // wasm-target crate in the workspace to check a list of sixteen strings.
    // The parse is deliberately narrow: a list it cannot find is a failure, not
    // a pass, so a rename cannot turn this check off by accident.
    //
    // What this deliberately does not check is whether a listed row has a
    // fixture. Five of the sixteen do not, which is a real finding and already
    // a counted gap on each of those rows. Whether the public face should offer
    // an unevidenced number is a decision about what the face is for — it shows
    // credibility beside every answer, so the number is not presented as more
    // than it is — and encoding an answer to that here would be this check
    // inventing policy rather than enforcing it.
    out.push(demonstration_subset(tree));

    out
}

/// V13, kept separate because it is the one assembly check that reads a file
/// outside the tree.
fn demonstration_subset(tree: &Tree) -> Check {
    const NAME: &str = "V13 the demonstration subset is answerable";
    let path = tree.root.join("crates/vleo-wasm/src/lib.rs");
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        // Not a pass. The face existing and this check not finding it is the
        // same situation as the face being wrong, from here.
        Err(e) => return Check::fail(NAME, format!("{}: {e}", path.display())),
    };
    let Some(start) = text.find("const DEMONSTRATION: &[&str] = &[") else {
        return Check::fail(
            NAME,
            format!(
                "{} has no `const DEMONSTRATION: &[&str] = &[` — if the list was renamed, rename \
                 it here too rather than leaving a check that silently passes",
                path.display()
            ),
        );
    };
    let body = &text[start..];
    let Some(end) = body.find("];") else {
        return Check::fail(NAME, "the DEMONSTRATION list is not terminated".into());
    };
    let listed: Vec<String> = body[..end]
        .lines()
        .filter_map(|l| {
            let l = l.trim();
            l.strip_prefix('"')
                .and_then(|l| l.split('"').next())
                .filter(|_| l.starts_with('"'))
                .map(str::to_string)
        })
        .collect();

    let mut bad = Vec::new();
    if listed.is_empty() {
        bad.push("the list is empty or did not parse".to_string());
    }
    for id in &listed {
        match tree.sheets.get(id) {
            None => bad.push(format!("{id} is offered and is not a row in the tree")),
            // A seeded row refuses by name when run. That refusal is correct
            // everywhere else in the tool and wrong here: this face exists to
            // be clicked by somebody who has installed nothing.
            Some(sh) if sh.state != "published" => bad.push(format!(
                "{id} is offered and its state is '{}' — it would refuse rather than answer",
                sh.state
            )),
            Some(_) => {}
        }
    }

    // The prose beside the list counts it. A count in a comment is the first
    // thing to go stale, and this is cheaper than noticing later.
    let spelled = [
        (0, "zero"),
        (1, "one"),
        (2, "two"),
        (3, "three"),
        (4, "four"),
        (5, "five"),
        (6, "six"),
        (7, "seven"),
        (8, "eight"),
        (9, "nine"),
        (10, "ten"),
        (11, "eleven"),
        (12, "twelve"),
        (13, "thirteen"),
        (14, "fourteen"),
        (15, "fifteen"),
        (16, "sixteen"),
        (17, "seventeen"),
        (18, "eighteen"),
        (19, "nineteen"),
        (20, "twenty"),
    ];
    if let Some((_, word)) = spelled.iter().find(|(n, _)| *n == listed.len()) {
        for (n, other) in spelled.iter() {
            if *n != listed.len() && text.contains(&format!("{other} of them")) {
                bad.push(format!(
                    "the file says '{other} of them' and the list holds {} — write '{word} of them'",
                    listed.len()
                ));
            }
        }
    }

    if bad.is_empty() {
        Check::pass(NAME)
    } else {
        Check::fail(NAME, bad.join(", "))
    }
}

/// Depth-first search for a cycle that no case declares. Returns the loop it
/// found, so the message names both ends rather than saying a cycle exists.
fn find_cycles(tree: &Tree, declared: &BTreeSet<String>) -> Vec<String> {
    let ids: Vec<&String> = tree.sheets.keys().collect();
    let mut colour: std::collections::BTreeMap<&str, u8> =
        ids.iter().map(|i| (i.as_str(), 0u8)).collect();
    let mut stack: Vec<&str> = Vec::new();

    fn walk<'a>(
        node: &'a str,
        tree: &'a Tree,
        declared: &BTreeSet<String>,
        colour: &mut std::collections::BTreeMap<&'a str, u8>,
        stack: &mut Vec<&'a str>,
    ) -> Option<Vec<String>> {
        colour.insert(node, 1);
        stack.push(node);
        if let Some(sh) = tree.sheets.get(node) {
            for i in &sh.inputs {
                // An edge may name a node or one member of a set a node
                // publishes, and the PRODUCER is the node either way. Resolving
                // only the first form made an edge through a member invisible
                // here, so a cycle that ran through one would not be detected —
                // and a cycle the resolver cannot see is a run that does not
                // terminate rather than a gate failure.
                let p = match tree.sheets.get(producer_of(&i.var)) {
                    Some(p) => p.id.as_str(),
                    None => continue,
                };
                if declared.contains(node) && declared.contains(p) {
                    continue; // a declared loop; the resolver relaxes it
                }
                match colour.get(p).copied().unwrap_or(0) {
                    0 => {
                        if let Some(c) = walk(p, tree, declared, colour, stack) {
                            return Some(c);
                        }
                    }
                    1 => {
                        let at = stack.iter().position(|x| *x == p).unwrap_or(0);
                        let mut loop_ = stack[at..]
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>();
                        loop_.push(p.to_string());
                        return Some(loop_);
                    }
                    _ => {}
                }
            }
        }
        stack.pop();
        colour.insert(node, 2);
        None
    }

    for id in ids {
        if colour.get(id.as_str()).copied().unwrap_or(0) == 0 {
            if let Some(c) = walk(id.as_str(), tree, declared, &mut colour, &mut stack) {
                return c;
            }
        }
    }
    Vec::new()
}

/// Whether an input's variable id names something the tree publishes.
///
/// A node id, the ordinary case — or `<node id>.<publish id>`, one member of a
/// set a node publishes. A node id never contains a dot, so the two cannot be
/// confused. This lives beside the assembly checks because the node-level
/// contract check resolves the same two forms and the two must agree: a variable
/// that one accepts and the other calls dangling would fail the gate on a graph
/// that assembles perfectly well.
fn var_resolves(tree: &Tree, var: &str) -> bool {
    match var.split_once('.') {
        Some((node, member)) => tree
            .sheets
            .get(node)
            .is_some_and(|p| p.publishes.iter().any(|pb| pb.id == member)),
        None => tree.sheets.contains_key(var),
    }
}

/// The node that produces a variable, whichever form the id takes.
///
/// `"sw_f107_design_long"` produces itself; `"l3_solar_interface.ap_hotday"` is
/// produced by `l3_solar_interface`. Anything walking the graph by EDGES rather
/// than by variables wants this — the cycle detector above all, because an edge
/// it cannot follow is a loop it cannot find.
use crate::text::producer_of;
