//! A saved result: what one run returned, and the inputs it ran on.
//!
//! A run is a question somebody asked, and its answer is worth keeping — to
//! look at again, to send to someone, to compare with the next one. So a run
//! can be saved as ONE CSV: every input it ran on, every value it returned with
//! its unit and credibility, every row it could not run and why, and the run's
//! identity — the chain hash that says exactly which engine, tree and inputs
//! produced it. It opens in any spreadsheet. Uploaded again, it is shown as it
//! was, WITHOUT RUNNING ANYTHING: a saved result is a record, and re-running it
//! on today's engine would be a different result under the old one's name.
//!
//! Its input rows are the case it ran on, so they can be loaded back as the
//! case — which is how a result from somebody else becomes a starting point.
//!
//! Every value is written twice: in the row's own display unit for a person,
//! and in SI for the tool, so a result read back is the same numbers bit for
//! bit, whatever the display rounded.
//!
//! Like the case, results are kept outside the repository. Git never sees one.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::inputs::{case_inputs, num, template};
use crate::tables::VARS;
use crate::Vleo;

/// What a result file declares itself to be.
pub const FORMAT: &str = "vleo-result/1";

/// One row of a result: an input it ran on, a value it returned, or a row it
/// could not run.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Row {
    pub id: String,
    pub name: String,
    /// As a person reads it, in `unit`.
    pub value: String,
    pub unit: String,
    /// Exact, when there is a value.
    pub si: Option<f64>,
    pub credibility: String,
    pub governing: String,
    /// For an input, `changed` or `default`; for a blocked row, why.
    pub note: String,
}

/// A saved result.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Saved {
    pub target: String,
    pub mode: String,
    /// When it was saved, as the saver's clock said.
    pub saved: String,
    /// A name somebody gave it, or nothing.
    pub name: String,
    pub chain: String,
    pub kernel: String,
    pub graph: String,
    pub case: String,
    pub data: Vec<String>,
    /// The inputs template it ran against — see `inputs::template`.
    pub template: String,
    pub ran: usize,
    pub blocked_count: usize,
    pub inputs: Vec<Row>,
    pub outputs: Vec<Row>,
    pub blocked: Vec<Row>,
    /// The recorded version of every node the run went through that has one,
    /// with the release that carried it: `(node, version, release)`.
    pub versions: Vec<(String, u32, String)>,
    /// Whether the result says which versions it rested on at all — a
    /// `#! versions` line, empty or not. One saved before the tool recorded
    /// beliefs has none, and nothing can be said about what has moved since.
    pub versions_known: bool,
    /// When the full values were dropped to keep the folder small, or
    /// nothing. A thinned result keeps what it answered, what it was run on
    /// and its chain — enough to say what it was and to run it again — and
    /// says it was thinned rather than passing for whole.
    pub thinned: String,
}

impl Saved {
    /// The target's own answer, when the run produced one.
    pub fn answer(&self) -> Option<&Row> {
        self.outputs.iter().find(|r| r.id == self.target)
    }
    /// How many inputs it ran at a value other than their default.
    pub fn changed(&self) -> usize {
        self.inputs.iter().filter(|r| r.note == "changed").count()
    }
    /// The inputs it ran on, in SI — to load back as the case.
    pub fn case_values(&self) -> Vec<(String, f64)> {
        self.inputs
            .iter()
            .filter(|r| r.note == "changed")
            .filter_map(|r| r.si.map(|v| (r.id.clone(), v)))
            .collect()
    }
    /// What a listing needs and no more: the record's identity, the inputs it
    /// changed, and its answer. Every other value and blocked row is left out.
    pub fn summary(&self) -> Saved {
        let mut s = self.clone();
        s.inputs.retain(|r| r.note == "changed");
        s.outputs = self.answer().cloned().into_iter().collect();
        s.blocked.clear();
        s
    }
    /// The summary, marked as what a thinned result keeps.
    pub fn thinned_on(&self, date: &str) -> Saved {
        let mut s = self.summary();
        s.thinned = date.to_string();
        s
    }
}

/// A result from a run: the run's values, and the inputs it ran at — the saved
/// case with whatever the run itself set on top, in SI.
pub fn from_run(r: &vleo_bus::Results, supply: &[(String, f64)], saved: &str, name: &str) -> Saved {
    let m = &r.manifest;
    let mut s = Saved {
        target: m.node.clone(),
        mode: m.mode.to_string(),
        saved: saved.to_string(),
        name: name.to_string(),
        chain: m.chain.clone(),
        kernel: m.kernel.clone(),
        graph: m.graph.clone(),
        case: m.case.clone(),
        data: m.data.clone(),
        template: template(),
        ran: m.ran,
        blocked_count: m.blocked_count,
        ..Default::default()
    };
    for i in case_inputs() {
        // The last value supplied wins, exactly as the run applied them.
        let si = supply
            .iter()
            .rev()
            .find(|(k, _)| k == i.id)
            .map(|(_, v)| *v)
            .unwrap_or(i.default);
        s.inputs.push(Row {
            id: i.id.to_string(),
            name: i.label.to_string(),
            value: num(i.shown(si)),
            unit: i.unit.to_string(),
            si: Some(si),
            note: if si != i.default {
                "changed"
            } else {
                "default"
            }
            .to_string(),
            ..Default::default()
        });
    }
    for v in &r.values {
        let (shown, unit) = match Vleo::find(&v.id) {
            Some(k) => {
                let u = VARS[k as usize].unit;
                (num(v.value / u.si_factor()), u.symbol())
            }
            None => (num(v.value), v.unit),
        };
        s.outputs.push(Row {
            id: v.id.clone(),
            name: v.label.clone(),
            value: shown,
            unit: unit.to_string(),
            si: Some(v.value),
            credibility: v.cred.governing_score().to_string(),
            governing: v.governing.to_string(),
            ..Default::default()
        });
    }
    for o in &s.outputs {
        if let Some((n, rel)) = node_version(&o.id) {
            s.versions.push((o.id.clone(), n, rel.to_string()));
        }
    }
    s.versions_known = true;
    for b in &r.blocked {
        s.blocked.push(Row {
            id: b.id.clone(),
            name: Vleo::find(&b.id)
                .map(|k| VARS[k as usize].label.to_string())
                .unwrap_or_default(),
            note: b.message.clone(),
            ..Default::default()
        });
    }
    s
}

/// A node's current recorded version and the release that carried it, as
/// this build of the tool knows it.
pub fn node_version(id: &str) -> Option<(u32, &'static str)> {
    crate::tables::NODE_VERSIONS
        .iter()
        .find(|(n, _, _)| *n == id)
        .map(|(_, v, r)| (*v, *r))
}

/// The nodes whose record has moved past the version a result ran on:
/// `(node, then, now)`. Each is a belief the result rested on that has since
/// broken, as far as this build knows.
///
/// A row the run went through with no recorded version then rested on a belief
/// nobody had written down: its first version since is version 0 moving to 1.
/// That is only known for a result that says which versions it rested on.
pub fn moved_since(s: &Saved) -> Vec<(String, u32, u32)> {
    if !s.versions_known {
        return Vec::new();
    }
    let recorded = s.versions.iter().map(|(id, n, _)| (id.as_str(), *n));
    let unrecorded = s
        .outputs
        .iter()
        .filter(|o| !s.versions.iter().any(|(id, _, _)| *id == o.id))
        .map(|o| (o.id.as_str(), 0));
    recorded
        .chain(unrecorded)
        .filter_map(|(id, then)| {
            node_version(id)
                .filter(|(now, _)| *now > then)
                .map(|(now, _)| (id.to_string(), then, now))
        })
        .collect()
}

/// How a moved belief is said: `…had no recorded belief when this ran, and is
/// now at version 1` or `…was at version 2 when this ran, and is now at 3`.
pub fn moved_words(then: u32, now: u32) -> String {
    if then == 0 {
        format!("had no recorded belief when this ran, and is now at version {now}")
    } else {
        format!("was at version {then} when this ran, and is now at {now}")
    }
}

// ---------------------------------------------------------------------------
// the CSV

fn field(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\"").replace(['\n', '\r'], " "))
    } else {
        s.to_string()
    }
}

fn meta(s: &str) -> String {
    s.replace(['\n', '\r'], " ")
}

/// A result as the CSV it is kept and sent as.
pub fn csv(s: &Saved) -> String {
    let mut o = String::new();
    o.push_str(
        "# VLEO multipayload — a saved result: what one run returned, and the inputs it ran on.\n",
    );
    o.push_str(
        "# Open it in any spreadsheet. Upload it on the Results page to see it again without\n",
    );
    o.push_str("# running anything; its input rows can be loaded back as the case.\n");
    o.push_str(
        "# `value` is in `unit`; `si` is the same number in SI, which is what the tool reads.\n",
    );
    o.push_str(&format!("#! result {FORMAT}\n"));
    for (k, v) in [
        ("target", s.target.as_str()),
        ("mode", s.mode.as_str()),
        ("saved", s.saved.as_str()),
        ("name", s.name.as_str()),
        ("chain", s.chain.as_str()),
        ("kernel", s.kernel.as_str()),
        ("graph", s.graph.as_str()),
        ("case", s.case.as_str()),
        ("template", s.template.as_str()),
    ] {
        o.push_str(&format!("#! {k} {}\n", meta(v)));
    }
    o.push_str(&format!("#! data {}\n", meta(&s.data.join(" "))));
    o.push_str(&format!(
        "#! ran {}\n#! blocked {}\n",
        s.ran, s.blocked_count
    ));
    if s.versions_known {
        o.push_str(&format!(
            "#! versions {}\n",
            if s.versions.is_empty() {
                // Said, not left out: none recorded is itself a record, and a
                // row's first version after this is a belief that moved.
                "none".to_string()
            } else {
                s.versions
                    .iter()
                    .map(|(id, n, r)| format!("{id}=v{n}@{r}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        ));
    }
    if !s.thinned.is_empty() {
        o.push_str(&format!("#! thinned {}\n", meta(&s.thinned)));
    }
    o.push_str("section,id,name,value,unit,si,credibility,governing,note\n");
    for (section, rows) in [
        ("input", &s.inputs),
        ("output", &s.outputs),
        ("blocked", &s.blocked),
    ] {
        for r in rows {
            o.push_str(&format!(
                "{section},{},{},{},{},{},{},{},{}\n",
                field(&r.id),
                field(&r.name),
                field(&r.value),
                field(&r.unit),
                r.si.map(num).unwrap_or_default(),
                field(&r.credibility),
                field(&r.governing),
                field(&r.note)
            ));
        }
    }
    o
}

/// Read a result back. Refuses what is not one, by what is missing.
pub fn read(text: &str) -> Result<Saved, String> {
    // Said first, before any row is looked at: a case CSV or a spreadsheet of
    // something else would otherwise be refused for its columns, which tells a
    // person nothing about what they uploaded.
    if !text.lines().any(|l| {
        l.trim_start()
            .trim_start_matches('\u{feff}')
            .starts_with("#! result")
    }) {
        return Err(
            "this is not a saved result: it has no `#! result` line. Save one from a run, or \
             with `vleo run <node> --save <file.csv>`"
                .into(),
        );
    }
    let mut s = Saved::default();
    let mut format_ok = false;
    let mut header: Option<Vec<String>> = None;
    for (n, raw) in text.lines().enumerate() {
        let t = raw.trim().trim_start_matches('\u{feff}');
        if let Some(m) = t.strip_prefix("#!") {
            let m = m.trim();
            let (k, v) = m.split_once(' ').unwrap_or((m, ""));
            let v = v.trim().to_string();
            match k {
                "result" => {
                    if v != FORMAT {
                        return Err(format!(
                            "this is a `{v}` result, and this tool reads `{FORMAT}`"
                        ));
                    }
                    format_ok = true;
                }
                "target" => s.target = v,
                "mode" => s.mode = v,
                "saved" => s.saved = v,
                "name" => s.name = v,
                "chain" => s.chain = v,
                "kernel" => s.kernel = v,
                "graph" => s.graph = v,
                "case" => s.case = v,
                "template" => s.template = v,
                "thinned" => s.thinned = v,
                "data" => s.data = v.split_whitespace().map(str::to_string).collect(),
                "ran" => s.ran = v.parse().unwrap_or(0),
                "blocked" => s.blocked_count = v.parse().unwrap_or(0),
                "versions" => {
                    s.versions_known = true;
                    s.versions = v
                        .split_whitespace()
                        .filter_map(|x| {
                            let (id, rest) = x.split_once("=v")?;
                            let (n, rel) = rest.split_once('@')?;
                            Some((id.to_string(), n.parse().ok()?, rel.to_string()))
                        })
                        .collect()
                }
                _ => {}
            }
            continue;
        }
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let cells = crate::inputs::split_csv(t);
        let Some(h) = &header else {
            let h: Vec<String> = cells
                .iter()
                .map(|c| c.trim().to_ascii_lowercase())
                .collect();
            if !h.iter().any(|c| c == "section") || !h.iter().any(|c| c == "id") {
                return Err(format!(
                    "line {}: the first row that is not a comment must name the columns — \
                     section, id, name, value, unit, si",
                    n + 1
                ));
            }
            header = Some(h);
            continue;
        };
        let col = |name: &str| {
            h.iter()
                .position(|c| c == name)
                .and_then(|k| cells.get(k))
                .map(|x| x.trim().to_string())
                .unwrap_or_default()
        };
        let si = col("si");
        let row = Row {
            id: col("id"),
            name: col("name"),
            value: col("value"),
            unit: col("unit"),
            si: if si.is_empty() {
                None
            } else {
                Some(
                    si.parse::<f64>()
                        .map_err(|_| format!("line {}: si '{si}' is not a number", n + 1))?,
                )
            },
            credibility: col("credibility"),
            governing: col("governing"),
            note: col("note"),
        };
        match col("section").as_str() {
            "input" => s.inputs.push(row),
            "output" => s.outputs.push(row),
            "blocked" => s.blocked.push(row),
            other => {
                return Err(format!(
                    "line {}: section '{other}' is not input, output or blocked",
                    n + 1
                ))
            }
        }
    }
    if !format_ok {
        return Err(
            "this is not a saved result: it has no `#! result` line. Save one from a run, or \
             with `vleo run <node> --save <file.csv>`"
                .into(),
        );
    }
    if s.target.is_empty() || header.is_none() {
        return Err("this result names no target, or has no rows".into());
    }
    Ok(s)
}

/// A file name for a result: when, what, and which chain — so two saves of the
/// same question on different inputs never collide, and a listing sorts by time.
pub fn file_name(s: &Saved) -> String {
    let when: String = s
        .saved
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    // The target and chain come from the file being kept, which may be an
    // upload. Only the characters a result's name is allowed to hold survive,
    // so a crafted `..\` never becomes a path on Windows (where `\` is one).
    let plain = |x: &str| -> String {
        x.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || matches!(c, '_' | '-') {
                    c
                } else {
                    '-'
                }
            })
            .collect()
    };
    format!(
        "{}_{}_{}.csv",
        when.trim_matches('-'),
        plain(&s.target),
        plain(&s.chain)
    )
}

// ---------------------------------------------------------------------------
// the report

fn he(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The same result as one HTML page that shows itself anywhere, for sending to
/// somebody who does not run the tool. The CSV rides inside it, so the page can
/// be uploaded in its place.
pub fn html(s: &Saved) -> String {
    let data = csv(s).replace("</", "<\\/");
    let answer = s
        .answer()
        .map(|a| {
            format!(
                "{} {}",
                he(&a.value),
                he(if a.unit == "-" { "" } else { &a.unit })
            )
        })
        .unwrap_or_else(|| "not computed on this run".to_string());
    let mut o = String::with_capacity(64 * 1024);
    o.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    o.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    o.push_str(&format!(
        "<title>{} — result</title>\n<style>\n{REPORT_CSS}</style>\n</head>\n<body>\n",
        he(&s.target)
    ));
    // ANSWER FIRST (docs/EXPLAINING.md E1): the number, then the three things
    // a reader needs before trusting it — how credible it is and what holds it
    // down, what could not run, and what it was run on. Then it said simply,
    // then where it breaks, then the reference tables.
    let target = s.answer();
    let label = target.map(|a| a.name.as_str()).unwrap_or(s.target.as_str());
    let changed_inputs: Vec<&Row> = s.inputs.iter().filter(|r| r.note == "changed").collect();
    let ran_line = format!(
        "<b>{} ran, {} blocked</b>{}.",
        s.ran,
        s.blocked_count,
        if s.blocked.is_empty() {
            String::new()
        } else {
            format!(
                " — {}",
                he(&s
                    .blocked
                    .iter()
                    .map(|b| b.id.as_str())
                    .take(4)
                    .collect::<Vec<_>>()
                    .join(", "))
            )
        }
    );
    let moved = moved_since(s);
    o.push_str(&format!(
        "<header><p class=\"k\">VLEO design tool · saved result</p><h1>{}</h1>\n\
         <section class=\"af\"><p class=\"afk\">Answer first</p><p class=\"answer\">{}</p>\n<ul>\
         <li>{}</li><li>{}</li><li>{}</li><li>{}</li></ul></section>\n\
         <p class=\"m\">{}saved {} · mode {} · chain <code>{}</code> · kernel <code>{}</code> · \
         graph <code>{}</code>{}. This page is a record: it shows what the run returned when it was \
         saved, and nothing here runs again.</p></header>\n",
        he(&s.target),
        answer,
        match target {
            Some(a) => format!(
                "Credibility <b>{} of 4</b>, held down by <b>{}</b> — the weakest of eight factors.",
                he(&a.credibility),
                he(&a.governing)
            ),
            None => "The target did not answer on this run; see what could not run.".to_string(),
        },
        ran_line,
        if changed_inputs.is_empty() {
            format!("Run on the declared defaults: none of {} inputs changed.", s.inputs.len())
        } else {
            format!(
                "Run on <b>{} of {}</b> inputs changed from their defaults: {}.",
                changed_inputs.len(),
                s.inputs.len(),
                he(&changed_inputs
                    .iter()
                    .take(4)
                    .map(|r| format!("{} = {} {}", r.id, r.value, if r.unit == "-" { "" } else { &r.unit }))
                    .collect::<Vec<_>>()
                    .join(", "))
            )
        },
        if !moved.is_empty() {
            format!(
                "<b>{} belief{} it rests on {} since broken</b> — see where it breaks.",
                moved.len(),
                if moved.len() == 1 { "" } else { "s" },
                if moved.len() == 1 { "has" } else { "have" }
            )
        } else if s.versions.is_empty() {
            "No node it ran through had a recorded belief when it was saved.".to_string()
        } else {
            format!(
                "It rests on {} recorded node version{}, none since replaced.",
                s.versions.len(),
                if s.versions.len() == 1 { "" } else { "s" }
            )
        },
        if s.name.is_empty() {
            String::new()
        } else {
            format!("<b>{}</b> · ", he(&s.name))
        },
        he(&s.saved),
        he(&s.mode),
        he(&s.chain),
        he(&s.kernel),
        he(&s.graph),
        if s.data.is_empty() {
            String::new()
        } else {
            format!(" · data {}", he(&s.data.join(", ")))
        }
    ));
    o.push_str(&format!(
        "<section><h2>Said simply <span class=\"dx\">explanation</span></h2><p>On {} inputs, \
         <b>{}</b> comes out at <b>{}</b>. Every number below was worked out by the tool from the \
         relations its rows cite <span class=\"claim\">derived</span>, on inputs that are either a \
         row's declared default <span class=\"claim\">declared</span> or a value this case set.</p></section>\n",
        if changed_inputs.is_empty() { "the declared" } else { "this case's" },
        he(label),
        answer
    ));
    o.push_str("<section><h2>Where it breaks <span class=\"dx\">explanation</span></h2><ul>");
    if let Some(a) = target {
        o.push_str(&format!(
            "<li>The answer is only as credible as its weakest factor: <b>{}</b>.</li>",
            he(&a.governing)
        ));
    }
    if s.blocked.is_empty() {
        o.push_str("<li>Nothing on its chain was blocked.</li>");
    } else {
        o.push_str(&format!(
            "<li>{} row{} could not run, and nothing that needed {} does — listed below with why.</li>",
            s.blocked.len(),
            if s.blocked.len() == 1 { "" } else { "s" },
            if s.blocked.len() == 1 { "it" } else { "them" }
        ));
    }
    for (id, then, now) in &moved {
        o.push_str(&format!(
            "<li><b><code>{}</code> {}</b>: a belief this result rested on has broken since. Run \
             it again to see what the new version says.</li>",
            he(id),
            moved_words(*then, *now)
        ));
    }
    if !s.versions.is_empty() {
        o.push_str(&format!(
            "<li>The node versions it rests on: {}.</li>",
            he(&s
                .versions
                .iter()
                .map(|(id, n, r)| format!("{id} v{n} ({r})"))
                .collect::<Vec<_>>()
                .join(", "))
        ));
    }
    o.push_str("</ul></section>\n");
    let table = |title: &str, rows: &[&Row], cols: &[&str], cells: &dyn Fn(&Row) -> Vec<String>| {
        let mut t = format!("<section><h2>{title} <span class=\"dx\">reference</span></h2>");
        if rows.is_empty() {
            t.push_str("<p class=\"m\">none</p></section>\n");
            return t;
        }
        t.push_str("<table><thead><tr>");
        for c in cols {
            t.push_str(&format!("<th>{c}</th>"));
        }
        t.push_str("</tr></thead><tbody>");
        for r in rows {
            t.push_str("<tr>");
            for c in cells(r) {
                t.push_str(&format!("<td>{c}</td>"));
            }
            t.push_str("</tr>");
        }
        t.push_str("</tbody></table></section>\n");
        t
    };
    let unit = |u: &str| he(if u == "-" { "" } else { u });
    let value_cells = |r: &Row| {
        vec![
            format!("<code>{}</code>", he(&r.id)),
            he(&r.name),
            format!("{} {}", he(&r.value), unit(&r.unit)),
            he(&r.credibility),
            he(&r.governing),
        ]
    };
    o.push_str(&table(
        "Inputs changed from their defaults",
        &changed_inputs,
        &["input", "name", "ran at"],
        &|r| {
            vec![
                format!("<code>{}</code>", he(&r.id)),
                he(&r.name),
                format!("{} {}", he(&r.value), unit(&r.unit)),
            ]
        },
    ));
    let outs: Vec<&Row> = s.outputs.iter().collect();
    o.push_str(&table(
        "Every value the run returned",
        &outs,
        &["row", "name", "value", "credibility", "held down by"],
        &value_cells,
    ));
    let blocked: Vec<&Row> = s.blocked.iter().collect();
    o.push_str(&table(
        "What could not run, and why",
        &blocked,
        &["row", "name", "why"],
        &|r| {
            vec![
                format!("<code>{}</code>", he(&r.id)),
                he(&r.name),
                he(&r.note),
            ]
        },
    ));
    let all: Vec<&Row> = s.inputs.iter().collect();
    o.push_str("<details><summary>every input it ran on</summary>");
    o.push_str(&table(
        "All inputs",
        &all,
        &["input", "name", "ran at", ""],
        &|r| {
            vec![
                format!("<code>{}</code>", he(&r.id)),
                he(&r.name),
                format!("{} {}", he(&r.value), unit(&r.unit)),
                he(&r.note),
            ]
        },
    ));
    o.push_str("</details>\n");
    o.push_str(&format!(
        "<!-- The result itself, as CSV. Upload this page on the Results page and this is what \
         is read. -->\n<script type=\"text/csv\" id=\"vleo-result\">\n{data}</script>\n</body>\n</html>\n"
    ));
    if !s.thinned.is_empty() {
        // Said at the top, not left to be noticed: the tables below hold the
        // answer and the changed inputs only.
        o = o.replacen(
            "</header>\n",
            &format!(
                "<p class=\"m\"><b>Thinned on {}:</b> only the answer and the inputs it changed are \
                 kept. Its chain says exactly what ran; run it again on the same tool to see every \
                 value.</p></header>\n",
                he(&s.thinned)
            ),
            1,
        );
    }
    o
}

/// The CSV inside a report page, or the text itself when it is not one.
pub fn unwrap_report(text: &str) -> String {
    let Some(at) = text.find("id=\"vleo-result\"") else {
        return text.to_string();
    };
    let Some(open) = text[at..].find('>').map(|k| at + k + 1) else {
        return text.to_string();
    };
    let close = text[open..]
        .find("</script>")
        .map(|k| open + k)
        .unwrap_or(text.len());
    text[open..close].replace("<\\/", "</")
}

const REPORT_CSS: &str = ":root { --paper:#faf9f5; --card:#fff; --ink:#1f1e1b; --ink2:#55524a; --rule:#dedbd2; --accent:#6b3fa0; }
@media (prefers-color-scheme: dark) { :root { --paper:#1b1a18; --card:#23221f; --ink:#ecebe6; --ink2:#b3afa5; --rule:#3a3833; --accent:#b99ae0; } }
body { margin: 0 auto; max-width: 980px; padding: 0 16px 48px; background: var(--paper); color: var(--ink);
  font: 15px/1.5 system-ui, -apple-system, 'Segoe UI', sans-serif; }
.k { font: 12px ui-monospace, Menlo, monospace; letter-spacing: .08em; text-transform: uppercase; color: var(--accent); margin: 28px 0 4px; }
h1 { font: 600 22px ui-monospace, Menlo, monospace; margin: 0; }
h2 { font-size: 16px; margin: 28px 0 6px; border-bottom: 1px solid var(--rule); padding-bottom: 4px; }
.answer { font-size: 30px; font-weight: 600; margin: 6px 0; }
.m { color: var(--ink2); font-size: 13px; margin: 4px 0; }
code { font: 12.5px ui-monospace, Menlo, monospace; }
table { width: 100%; border-collapse: collapse; font-size: 13px; background: var(--card); }
th, td { text-align: left; padding: 4px 8px; border-bottom: 1px solid var(--rule); vertical-align: top; overflow-wrap: anywhere; }
th { font-weight: 500; color: var(--ink2); }
details { margin-top: 24px; }
.af { border-left: 3px solid var(--accent); padding: 4px 14px 8px; margin: 10px 0; background: var(--card); }
.afk { font: 11px ui-monospace, Menlo, monospace; letter-spacing: .08em; text-transform: uppercase; color: var(--accent); margin: 6px 0 0; }
.af ul { margin: 4px 0; padding-left: 18px; font-size: 14px; }
.dx, .claim { font: 10.5px ui-monospace, Menlo, monospace; border: 1px solid var(--rule); border-radius: 3px; padding: 0 5px; color: var(--ink2); font-weight: 400; vertical-align: 2px; }
";

// ---------------------------------------------------------------------------
// kept on disk

/// Results kept in a directory outside the repository — on one laptop, or in
/// a folder the team shares.
///
/// Each result is its own file, and everything the folder knows about a
/// result sits in files named after it: a summary a listing reads instead of
/// the whole result, and a pin. There is no shared index file, so two
/// machines saving into one shared folder never write the same file, and a
/// summary lost or left stale is rebuilt from the result it summarises.
///
/// ```text
/// <dir>/<result>.csv            the result — whole, or thinned
/// <dir>/.index/<result>.head    its summary, rebuilt when missing or older
/// <dir>/.index/<result>.pin     present: kept whole, whatever its age
/// ```
#[cfg(feature = "std")]
pub mod store {
    use super::*;
    use std::path::{Path, PathBuf};

    /// Where the summaries and pins are kept, inside the results folder.
    pub const INDEX: &str = ".index";

    /// What a directory holds: each result's summary by its file name, and
    /// each file that does not read as one, with why.
    pub type Listing = (Vec<(String, Saved)>, Vec<(String, String)>);

    fn head_path(dir: &Path, name: &str) -> PathBuf {
        dir.join(INDEX).join(format!("{name}.head"))
    }

    fn pin_path(dir: &Path, name: &str) -> PathBuf {
        dir.join(INDEX).join(format!("{name}.pin"))
    }

    fn modified(p: &Path) -> Option<std::time::SystemTime> {
        std::fs::metadata(p).and_then(|m| m.modified()).ok()
    }

    /// Write a result's summary beside it. A folder that cannot take it — read
    /// only, say — costs only speed: the listing reads the whole result instead.
    fn write_head(dir: &Path, name: &str, s: &Saved) {
        if std::fs::create_dir_all(dir.join(INDEX)).is_ok() {
            let _ = crate::files::write_whole(&head_path(dir, name), csv(&s.summary()));
        }
    }

    /// Every result in the directory, newest first, as its summary — the
    /// answer, the inputs it changed and its identity, without reading every
    /// value of every result. Any file that no longer reads is named rather
    /// than skipped.
    pub fn list(dir: &Path) -> Listing {
        let mut good = Vec::new();
        let mut bad = Vec::new();
        let Ok(rd) = std::fs::read_dir(dir) else {
            return (good, bad);
        };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".csv") || name.starts_with('.') {
                continue;
            }
            // The summary, when it is at least as new as the result: a result
            // replaced by another machine since its summary was written is
            // read whole again.
            let head = head_path(dir, &name);
            let fresh = match (modified(&head), modified(&e.path())) {
                (Some(h), Some(f)) => h >= f,
                _ => false,
            };
            if fresh {
                if let Ok(s) = std::fs::read_to_string(&head)
                    .map_err(|x| x.to_string())
                    .and_then(|t| read(&t))
                {
                    good.push((name, s));
                    continue;
                }
            }
            match std::fs::read_to_string(e.path())
                .map_err(|x| x.to_string())
                .and_then(|t| read(&t))
            {
                Ok(s) => {
                    write_head(dir, &name, &s);
                    good.push((name, s.summary()));
                }
                Err(why) => bad.push((name, why)),
            }
        }
        good.sort_by(|a, b| b.0.cmp(&a.0));
        (good, bad)
    }

    /// Keep a result. Returns the file name it is kept under.
    pub fn save(dir: &Path, s: &Saved) -> Result<String, String> {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let name = file_name(s);
        // The same rule `open` and `remove` hold a name to, applied before the
        // write rather than trusted from `file_name`.
        if !is_plain(&name) {
            return Err(format!("'{name}' is not a result's file name"));
        }
        crate::files::write_whole(&dir.join(&name), csv(s)).map_err(|e| format!("{name}: {e}"))?;
        write_head(dir, &name, s);
        Ok(name)
    }

    /// One result by its file name. The name must be a plain file name in the
    /// directory: a path read from a request never reaches anywhere else.
    pub fn open(dir: &Path, name: &str) -> Result<Saved, String> {
        if !is_plain(name) {
            return Err(format!("'{name}' is not a result's file name"));
        }
        let text = std::fs::read_to_string(dir.join(name)).map_err(|e| format!("{name}: {e}"))?;
        read(&text)
    }

    /// Remove one result, with its summary and its pin.
    pub fn remove(dir: &Path, name: &str) -> Result<(), String> {
        if !is_plain(name) {
            return Err(format!("'{name}' is not a result's file name"));
        }
        std::fs::remove_file(dir.join(name)).map_err(|e| format!("{name}: {e}"))?;
        let _ = std::fs::remove_file(head_path(dir, name));
        let _ = std::fs::remove_file(pin_path(dir, name));
        Ok(())
    }

    /// Whether a result is pinned: kept whole whatever its age.
    pub fn pinned(dir: &Path, name: &str) -> bool {
        is_plain(name) && pin_path(dir, name).exists()
    }

    /// Pin a result, or unpin it. A pin is its own file, so pinning on one
    /// machine never rewrites anything another machine is reading.
    pub fn pin(dir: &Path, name: &str, on: bool) -> Result<(), String> {
        if !is_plain(name) || !dir.join(name).is_file() {
            return Err(format!("'{name}' is not a result kept here"));
        }
        if on {
            std::fs::create_dir_all(dir.join(INDEX)).map_err(|e| format!("{INDEX}: {e}"))?;
            crate::files::write_whole(&pin_path(dir, name), "pinned\n")
                .map_err(|e| format!("{name}: {e}"))
        } else {
            match std::fs::remove_file(pin_path(dir, name)) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(format!("{name}: {e}")),
                _ => Ok(()),
            }
        }
    }

    /// Thin every result saved before `before` (a `YYYY-MM-DD` date) that is
    /// neither pinned nor thinned already: keep its answer, the inputs it
    /// changed and its identity, and drop every other value. Returns the
    /// names thinned. The inputs it changed are its case, so a thinned result
    /// can always be run again; what is dropped is only what that run would
    /// return.
    pub fn thin(dir: &Path, before: &str, today: &str) -> Vec<String> {
        let mut done = Vec::new();
        for (name, head) in list(dir).0 {
            if !head.thinned.is_empty()
                || head.saved.get(..10).is_none_or(|d| d >= before)
                || pinned(dir, &name)
            {
                continue;
            }
            let Ok(whole) = open(dir, &name) else {
                continue;
            };
            let thin = whole.thinned_on(today);
            if crate::files::write_whole(&dir.join(&name), csv(&thin)).is_ok() {
                write_head(dir, &name, &thin);
                done.push(name);
            }
        }
        done
    }

    fn is_plain(name: &str) -> bool {
        name.ends_with(".csv")
            && !name.is_empty()
            && !name.starts_with('.')
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
            && !name.contains("..")
    }
}

/// A result as one file to send: `.vleo`, a zip of the result's CSV, its
/// report page and a manifest that says what it is. Any unzip tool opens it;
/// the Results page takes it back whole.
#[cfg(feature = "std")]
pub mod share {
    use super::*;

    /// What a share file declares itself to be.
    pub const FORMAT: &str = "vleo-share/1";

    /// The share file of a result.
    pub fn pack(s: &Saved) -> Vec<u8> {
        let manifest = format!(
            "# A VLEO result, to send. Open report.html in a browser; result.csv in a spreadsheet.\n\
             format = \"{FORMAT}\"\nresult = \"result.csv\"\nreport = \"report.html\"\n\
             target = \"{}\"\nsaved = \"{}\"\nchain = \"{}\"\nthinned = {}\n",
            toml_str(&s.target),
            toml_str(&s.saved),
            toml_str(&s.chain),
            !s.thinned.is_empty()
        );
        crate::files::zip(&[
            ("manifest.toml", manifest.as_bytes()),
            ("result.csv", csv(s).as_bytes()),
            ("report.html", html(s).as_bytes()),
        ])
    }

    /// The result inside a share file.
    pub fn unpack(bytes: &[u8]) -> Result<Saved, String> {
        let files =
            crate::files::unzip(bytes).map_err(|e| format!("this is not a .vleo file: {e}"))?;
        let get = |n: &str| files.iter().find(|(k, _)| k == n).map(|(_, v)| v);
        let manifest = get("manifest.toml")
            .map(|b| String::from_utf8_lossy(b).into_owned())
            .ok_or("this .vleo file has no manifest.toml")?;
        if !manifest.contains(&format!("format = \"{FORMAT}\"")) {
            return Err(format!("this .vleo file is not `{FORMAT}`"));
        }
        let text = get("result.csv").ok_or("this .vleo file has no result.csv")?;
        read(&String::from_utf8_lossy(text))
    }

    fn toml_str(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace(['\n', '\r'], " ")
    }
}
