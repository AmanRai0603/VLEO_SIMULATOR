//! A lesson: `lesson.toml` beside a row's `node.toml`, written by the expert
//! who knows the row, and drawn by the page from the component library.
//!
//! NOBODY WRITES HTML FOR A PAGE (docs/ARCHITECTURE.html, section 5). A lesson
//! is content — stations of text with a claim tag each, equations with their
//! source, "try it" widgets declared as *these input rows, these output rows*,
//! check-yourself questions, references — and the page builds every part of
//! it from a named component. So a lesson may not carry markup, and it may not
//! carry a formula to compute: a widget names rows, and the engine computes
//! them, so a lesson cannot disagree with the tool.
//!
//! Outside the sheet hash, like `[explain]` and `[theory]`: a lesson is how a
//! row is taught, not what it computes. The gate reads it — every key known,
//! every claim tagged, every row it names real — and refuses one that fails.

use std::path::Path;

use crate::load::Tree;

/// The file a lesson lives in, beside `node.toml`.
pub const FILE: &str = "lesson.toml";

/// The kinds of reading a lesson may be (docs/EXPLAINING.md).
pub const KINDS: &[&str] = &["tutorial", "how-to", "explanation", "reference"];
/// Where a station sits in the explaining order: said simply, the real thing,
/// where it breaks — and a story, which may sit anywhere.
pub const STATIONS: &[&str] = &["simply", "real", "breaks", "story"];
/// What a claim rests on.
pub const CLAIMS: &[&str] = &["sourced", "derived", "declared", "illustrative"];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lesson {
    /// The row it teaches: the folder it sits in.
    pub node: String,
    pub title: String,
    /// Who wrote it — a person, as on a form.
    pub by: String,
    /// Answer first: the one sentence a reader takes away.
    pub answer: String,
    pub kind: String,
    pub stations: Vec<Station>,
    pub equations: Vec<Equation>,
    pub widgets: Vec<Widget>,
    pub checks: Vec<Question>,
    pub references: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Station {
    pub kind: String,
    pub title: String,
    pub text: String,
    pub claim: String,
    pub source: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Equation {
    /// As a reader sees it, in plain text: `v = sqrt(mu / r)`.
    pub text: String,
    pub says: String,
    pub claim: String,
    pub source: String,
}

/// A "try it" widget: sliders for these input rows, and the engine's answer
/// for these output rows. With `sweep`, the first output is also drawn across
/// that input's range.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Widget {
    pub title: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub sweep: String,
}

/// A check-yourself question: its options, which one is right, and why.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Question {
    pub question: String,
    pub options: Vec<String>,
    /// Counted from 1, as a person counts.
    pub answer: usize,
    pub why: String,
}

/// The lesson in a row's folder: `None` when it has none, and the reason when
/// the file does not read.
pub fn load(dir: &Path, node: &str) -> Option<Result<Lesson, String>> {
    let p = dir.join(FILE);
    if !p.is_file() {
        return None;
    }
    Some(
        std::fs::read_to_string(&p)
            .map_err(|e| format!("{}: {e}", p.display()))
            .and_then(|t| read(&t, node).map_err(|e| format!("{}: {e}", p.display()))),
    )
}

/// A lesson from its text. Every key is one this reader knows: a misspelt key
/// is refused, not ignored, because an ignored `sourse` is a claim that looks
/// sourced and is not.
pub fn read(text: &str, node: &str) -> Result<Lesson, String> {
    let doc: toml::Table = text.parse().map_err(|e| format!("not TOML: {e}"))?;
    known(
        &doc,
        "",
        &[
            "lesson",
            "station",
            "equation",
            "widget",
            "check",
            "reference",
        ],
    )?;
    let head = doc
        .get("lesson")
        .and_then(|v| v.as_table())
        .ok_or("there is no [lesson] table")?;
    known(head, "[lesson]", &["title", "by", "answer", "kind"])?;
    let s = |t: &toml::Table, k: &str| t.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
    let list = |t: &toml::Table, k: &str| -> Vec<String> {
        t.get(k)
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    let tables = |k: &str| -> Result<Vec<toml::Table>, String> {
        match doc.get(k) {
            None => Ok(Vec::new()),
            Some(v) => v
                .as_array()
                .ok_or(format!("[[{k}]] must be a list of tables"))?
                .iter()
                .map(|t| {
                    t.as_table()
                        .cloned()
                        .ok_or(format!("[[{k}]] must be a list of tables"))
                })
                .collect(),
        }
    };
    let mut l = Lesson {
        node: node.to_string(),
        title: s(head, "title"),
        by: s(head, "by"),
        answer: s(head, "answer"),
        kind: s(head, "kind"),
        ..Default::default()
    };
    for t in tables("station")? {
        known(
            &t,
            "[[station]]",
            &["kind", "title", "text", "claim", "source"],
        )?;
        l.stations.push(Station {
            kind: s(&t, "kind"),
            title: s(&t, "title"),
            text: s(&t, "text"),
            claim: s(&t, "claim"),
            source: s(&t, "source"),
        });
    }
    for t in tables("equation")? {
        known(&t, "[[equation]]", &["text", "says", "claim", "source"])?;
        l.equations.push(Equation {
            text: s(&t, "text"),
            says: s(&t, "says"),
            claim: s(&t, "claim"),
            source: s(&t, "source"),
        });
    }
    for t in tables("widget")? {
        known(&t, "[[widget]]", &["title", "inputs", "outputs", "sweep"])?;
        l.widgets.push(Widget {
            title: s(&t, "title"),
            inputs: list(&t, "inputs"),
            outputs: list(&t, "outputs"),
            sweep: s(&t, "sweep"),
        });
    }
    for t in tables("check")? {
        known(&t, "[[check]]", &["question", "options", "answer", "why"])?;
        let answer = match t.get("answer") {
            None => 0,
            Some(v) => v
                .as_integer()
                .filter(|n| *n >= 1)
                .ok_or("[[check]] answer is the number of the right option, counted from 1")?
                as usize,
        };
        l.checks.push(Question {
            question: s(&t, "question"),
            options: list(&t, "options"),
            answer,
            why: s(&t, "why"),
        });
    }
    for t in tables("reference")? {
        known(&t, "[[reference]]", &["text"])?;
        l.references.push(s(&t, "text"));
    }
    Ok(l)
}

fn known(t: &toml::Table, at: &str, keys: &[&str]) -> Result<(), String> {
    let bad: Vec<&str> = t
        .keys()
        .map(String::as_str)
        .filter(|k| !keys.contains(k))
        .collect();
    if bad.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{}{} not a key a lesson has — {}",
            if at.is_empty() {
                String::new()
            } else {
                format!("{at}: ")
            },
            bad.iter()
                .map(|k| format!("`{k}`"))
                .collect::<Vec<_>>()
                .join(", "),
            if at.is_empty() {
                "a lesson has [lesson], [[station]], [[equation]], [[widget]], [[check]] and [[reference]]".to_string()
            } else {
                format!("it has {}", keys.join(", "))
            }
        ))
    }
}

/// Everything wrong with a lesson, against the tree it teaches. Empty when it
/// may be drawn.
pub fn problems(l: &Lesson, tree: &Tree) -> Vec<String> {
    let mut out = Vec::new();
    for (k, v) in [("title", &l.title), ("by", &l.by), ("answer", &l.answer)] {
        if v.trim().is_empty() {
            out.push(format!("[lesson] {k} is empty"));
        }
    }
    if !KINDS.contains(&l.kind.as_str()) {
        out.push(format!(
            "[lesson] kind '{}' is not one of {}",
            l.kind,
            KINDS.join(", ")
        ));
    }
    if l.stations.is_empty() {
        out.push("a lesson with no [[station]] teaches nothing".into());
    }
    let claim = |at: &str, c: &str, src: &str, out: &mut Vec<String>| {
        if !CLAIMS.contains(&c) {
            out.push(format!(
                "{at}: claim '{c}' is not one of {}",
                CLAIMS.join(", ")
            ));
        } else if c == "sourced" && src.trim().is_empty() {
            out.push(format!("{at}: a sourced claim names its source"));
        }
    };
    let mut texts: Vec<(String, &str)> = vec![("[lesson] answer".into(), &l.answer)];
    for (i, s) in l.stations.iter().enumerate() {
        let at = format!("station {}", i + 1);
        if !STATIONS.contains(&s.kind.as_str()) {
            out.push(format!(
                "{at}: kind '{}' is not one of {}",
                s.kind,
                STATIONS.join(", ")
            ));
        }
        if s.text.trim().is_empty() {
            out.push(format!("{at}: text is empty"));
        }
        claim(&at, &s.claim, &s.source, &mut out);
        texts.push((at.clone(), &s.text));
        texts.push((at, &s.title));
    }
    for (i, e) in l.equations.iter().enumerate() {
        let at = format!("equation {}", i + 1);
        if e.text.trim().is_empty() || e.says.trim().is_empty() {
            out.push(format!("{at}: an equation has its text and what it says"));
        }
        claim(&at, &e.claim, &e.source, &mut out);
        texts.push((at.clone(), &e.text));
        texts.push((at, &e.says));
    }
    for (i, w) in l.widgets.iter().enumerate() {
        let at = format!("widget {}", i + 1);
        if w.inputs.is_empty() || w.outputs.is_empty() {
            out.push(format!(
                "{at}: a widget names the input rows a reader moves and the rows it shows"
            ));
        }
        for id in &w.inputs {
            match tree.sheets.get(id) {
                None => out.push(format!("{at}: input '{id}' is not a row")),
                Some(sh) if !sh.is_declared() => out.push(format!(
                    "{at}: input '{id}' is computed, not declared — a reader can move only a value a person picks"
                )),
                _ => {}
            }
        }
        for id in &w.outputs {
            match tree.sheets.get(id) {
                None => out.push(format!("{at}: output '{id}' is not a row")),
                Some(sh) if sh.is_seeded() => out.push(format!(
                    "{at}: output '{id}' is seeded — it has nothing to compute yet"
                )),
                _ => {}
            }
        }
        if !w.sweep.is_empty() && !w.inputs.contains(&w.sweep) {
            out.push(format!(
                "{at}: sweep '{}' is not one of its inputs",
                w.sweep
            ));
        }
    }
    for (i, q) in l.checks.iter().enumerate() {
        let at = format!("check {}", i + 1);
        if q.question.trim().is_empty() || q.why.trim().is_empty() {
            out.push(format!(
                "{at}: a check asks a question and says why its answer is right"
            ));
        }
        if q.options.len() < 2 {
            out.push(format!("{at}: a check offers at least two options"));
        } else if q.answer == 0 || q.answer > q.options.len() {
            out.push(format!(
                "{at}: answer {} is not one of its {} options",
                q.answer,
                q.options.len()
            ));
        }
        texts.push((at.clone(), &q.question));
        texts.push((at, &q.why));
    }
    // NO MARKUP. The page builds every part from a component; a tag in the
    // content is a component built by hand, and a script is worse.
    for (at, t) in texts {
        if let Some(tag) = markup(t) {
            out.push(format!(
                "{at}: '{tag}' is markup; a lesson is text, and the page draws it"
            ));
        }
    }
    out
}

/// The first thing in `t` that reads as an HTML tag, if any.
fn markup(t: &str) -> Option<String> {
    let b = t.as_bytes();
    (0..b.len()).find_map(|i| {
        let next = *b.get(i + 1)?;
        (b[i] == b'<' && (next.is_ascii_alphabetic() || next == b'/' || next == b'!')).then(|| {
            t[i..]
                .chars()
                .take_while(|c| *c != '>')
                .take(24)
                .collect::<String>()
                + ">"
        })
    })
}

/// A lesson as JSON, for the page.
pub fn json(l: &Lesson) -> String {
    let q = |s: &str| {
        let mut o = String::from("\"");
        for c in s.chars() {
            match c {
                '"' => o.push_str("\\\""),
                '\\' => o.push_str("\\\\"),
                '\n' => o.push_str("\\n"),
                '\r' => o.push_str("\\r"),
                '\t' => o.push_str("\\t"),
                c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
                c => o.push(c),
            }
        }
        o.push('"');
        o
    };
    let arr = |v: Vec<String>| format!("[{}]", v.join(","));
    let strs = |v: &[String]| arr(v.iter().map(|s| q(s)).collect());
    format!(
        "{{\"node\":{},\"title\":{},\"by\":{},\"answer\":{},\"kind\":{},\"stations\":{},\
         \"equations\":{},\"widgets\":{},\"checks\":{},\"references\":{}}}",
        q(&l.node),
        q(&l.title),
        q(&l.by),
        q(&l.answer),
        q(&l.kind),
        arr(l
            .stations
            .iter()
            .map(|s| format!(
                "{{\"kind\":{},\"title\":{},\"text\":{},\"claim\":{},\"source\":{}}}",
                q(&s.kind),
                q(&s.title),
                q(&s.text),
                q(&s.claim),
                q(&s.source)
            ))
            .collect()),
        arr(l
            .equations
            .iter()
            .map(|e| format!(
                "{{\"text\":{},\"says\":{},\"claim\":{},\"source\":{}}}",
                q(&e.text),
                q(&e.says),
                q(&e.claim),
                q(&e.source)
            ))
            .collect()),
        arr(l
            .widgets
            .iter()
            .map(|w| format!(
                "{{\"title\":{},\"inputs\":{},\"outputs\":{},\"sweep\":{}}}",
                q(&w.title),
                strs(&w.inputs),
                strs(&w.outputs),
                q(&w.sweep)
            ))
            .collect()),
        arr(l
            .checks
            .iter()
            .map(|c| format!(
                "{{\"question\":{},\"options\":{},\"answer\":{},\"why\":{}}}",
                q(&c.question),
                strs(&c.options),
                c.answer,
                q(&c.why)
            ))
            .collect()),
        strs(&l.references)
    )
}
