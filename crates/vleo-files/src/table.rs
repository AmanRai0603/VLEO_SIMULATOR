//! A table pasted from a spreadsheet, or a CSV file, read as a node's cases
//! (docs/briefs/F8.md, F8-b item 1; questions 72 and 73).
//!
//! What a spreadsheet copies is tab-separated, a row a line; a CSV file is
//! comma-separated, read by [`crate::csv`]. The first row names the columns
//! as the node's cases do: each input by its port's name, `answer`, an
//! `answer.<member>` for each value published beside it, `tolerance`,
//! `refuses`, `says` and `origin`; each with its unit in square brackets,
//! as `lead [d]`, or none, when it is the port's.
//!
//! Every number is read by the method language's own amount reader
//! (`vleo_sheet::method::amount`), in its column's unit, and said back; an
//! input's or the answer's is then said in its port's unit, exactly, since
//! signing needs the port's unit in the header (question 73). A cell that
//! does not read is refused by its line and column, with the fix, and
//! nothing is guessed: no unit, no tolerance and no origin is filled in. A
//! refusal (`refuses` yes) may leave its answer and tolerance empty, and an
//! input it refuses may be `NaN` or an infinity, as the checks read one.
//!
//! The cases are the one external oracle (rule 4), so where each expected
//! value came from is the person's to say: once for the whole paste, or in
//! the table's own `origin` column, never both and never defaulted
//! (question 72).

use crate::checks;
use crate::csv;
use crate::model::Port;
use vleo_sheet::method::amount;

/// One cell read: where it is, what was typed, and what it was read as.
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    /// The line it is on, as the person sees the paste: the header is 1.
    pub line: usize,
    pub column: String,
    pub typed: String,
    pub said: String,
}

/// A paste read as cases: the `results/isolation.csv` text it becomes, its
/// number of cases, and each number cell said back.
#[derive(Clone, Debug, PartialEq)]
pub struct Read {
    pub csv: String,
    pub cases: usize,
    pub cells: Vec<Cell>,
}

/// The columns a case may have beside its inputs.
const OTHERS: &[&str] = &["answer", "tolerance", "refuses", "says", "origin"];

/// The rows of `pasted`: tab-separated when its first line holds a tab, as a
/// spreadsheet copies; else read as CSV. Each row with its line.
fn rows_of(pasted: &str) -> Result<Vec<(usize, Vec<String>)>, String> {
    let text = pasted.replace('\r', "");
    let first = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    if first.contains('\t') {
        return Ok(text
            .lines()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty())
            .map(|(k, l)| (k + 1, l.split('\t').map(|c| c.trim().to_string()).collect()))
            .collect());
    }
    let p = csv::read(&text);
    if let Some((line, why)) = p.problems.first() {
        return Err(format!(
            "the table does not read as CSV, line {line}: {why}"
        ));
    }
    let mut out = vec![(1, p.head.clone())];
    for (k, r) in p.rows.into_iter().enumerate() {
        out.push((p.lines.get(k).copied().unwrap_or(k + 2), r));
    }
    Ok(out)
}

/// `pasted` read as the cases of a node whose inputs are `inputs` and whose
/// answer is `output`'s, each case's origin `origin` unless the table says
/// its own: refused by name, with the fix, at the first cell that does not
/// read, a column no case has, an input with no column, a missing
/// tolerance, or no origin said once.
pub fn cases(pasted: &str, inputs: &[&Port], output: &Port, origin: &str) -> Result<Read, String> {
    let rows = rows_of(pasted)?;
    let Some(((_, head), body)) = rows.split_first() else {
        return Err(
            "nothing is pasted: copy the table with its header row, as lead [d] and answer [sfu]"
                .into(),
        );
    };
    if body.is_empty() {
        return Err("the table has a header and no case: paste its rows with it".into());
    }
    // A spreadsheet copies empty columns at the end of a row: they are
    // dropped, never read as columns.
    let width = head
        .iter()
        .rposition(|h| !h.trim().is_empty())
        .map_or(0, |i| i + 1);
    let head: Vec<(String, String)> = head[..width]
        .iter()
        .map(|h| {
            let (n, u) = csv::split_unit(h);
            (n.to_string(), u.to_string())
        })
        .collect();
    let input = |n: &str| inputs.iter().find(|p| p.name == n);
    for (n, _) in &head {
        let known = input(n).is_some()
            || OTHERS.contains(&n.as_str())
            || n.strip_prefix("answer.").is_some_and(|m| !m.is_empty());
        if !known {
            return Err(format!(
                "the table has a column {n:?} no case has: its columns are the inputs {}, then answer, tolerance, refuses, says and origin",
                inputs.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ")
            ));
        }
        if head.iter().filter(|(m, _)| m == n).count() > 1 {
            return Err(format!("the table has two columns {n:?}: give each once"));
        }
    }
    for p in inputs {
        if !head.iter().any(|(n, _)| *n == p.name) {
            return Err(format!(
                "the table has no column for the input {}: every case says each input",
                p.name
            ));
        }
    }
    let has = |n: &str| head.iter().any(|(m, _)| m == n);
    if !has("answer") {
        return Err(
            "the table has no answer column: every case says what it expects, or refuses".into(),
        );
    }
    if !has("tolerance") {
        return Err("the table has no tolerance column: say how close each answer must be, as 1e-9 for one part in a billion".into());
    }
    let own = has("origin");
    match (own, origin.trim()) {
        (true, "") => {}
        (true, _) => {
            return Err("the table has its own origin column: leave the origin for the paste unset, or take the column out".into())
        }
        (false, "") => {
            return Err(format!(
                "say where these cases' expected values come from: {}; it is never filled in for you",
                checks::ORIGINS.join(", ")
            ))
        }
        (false, o) if !checks::ORIGINS.contains(&o) => {
            return Err(format!(
                "{o:?} is not where an expected value comes from: {}",
                checks::ORIGINS.join(", ")
            ))
        }
        _ => {}
    }
    // The header written: an input's and the answer's in their port's unit,
    // a member's in its own, the rest as they are; then the origin.
    let unit_of = |n: &str, typed: &str| -> String {
        match input(n) {
            Some(p) => p.unit.trim().to_string(),
            None if n == "answer" => output.unit.trim().to_string(),
            None => typed.to_string(),
        }
    };
    // A port's unit is written as it is, `[1]` for a pure number among them,
    // since signing compares the header's with the port's.
    let with_unit = |n: &str, u: &str| {
        if u.is_empty() {
            n.to_string()
        } else {
            format!("{n} [{u}]")
        }
    };
    let mut written: Vec<String> = head
        .iter()
        .map(|(n, u)| with_unit(n, &unit_of(n, u)))
        .collect();
    if !own {
        written.push("origin".into());
    }
    let refuses_at = head.iter().position(|(n, _)| n == "refuses");
    let mut cells = Vec::new();
    let mut out = Vec::new();
    for (line, row) in body {
        let cell = |i: usize| row.get(i).map_or("", |c| c.trim());
        if row.len() > width && row[width..].iter().any(|c| !c.trim().is_empty()) {
            return Err(format!(
                "line {line} has more cells than the header names: give each column a header"
            ));
        }
        let refuses = refuses_at.is_some_and(|i| cell(i).eq_ignore_ascii_case("yes"));
        let mut r = Vec::new();
        for (i, (n, typed_unit)) in head.iter().enumerate() {
            let typed = cell(i);
            let numeric =
                input(n).is_some() || n == "answer" || n.starts_with("answer.") || n == "tolerance";
            if !numeric {
                if n == "origin" && !checks::ORIGINS.contains(&typed) {
                    return Err(format!(
                        "line {line}: the origin {typed:?} is not where an expected value comes from: {}",
                        checks::ORIGINS.join(", ")
                    ));
                }
                r.push(typed.to_string());
                continue;
            }
            // A refusal expects no answer and no tolerance, which may be
            // empty; and an input it refuses may be no number at all, as the
            // checks read one (`NaN`, `inf`), kept as it is typed.
            if refuses {
                let answer = n == "answer" || n.starts_with("answer.") || n == "tolerance";
                let not_a_number = matches!(
                    typed.to_lowercase().as_str(),
                    "nan" | "inf" | "+inf" | "-inf" | "infinity" | "-infinity"
                );
                if (typed.is_empty() && answer) || (not_a_number && input(n).is_some()) {
                    r.push(typed.to_string());
                    continue;
                }
            }
            let read_in = if typed_unit.is_empty() {
                unit_of(n, "")
            } else {
                typed_unit.clone()
            };
            let a =
                amount::read(typed, &read_in).map_err(|why| format!("line {line}, {n}: {why}"))?;
            let port_unit = unit_of(n, typed_unit);
            let value = amount::value_in(&a, &port_unit)
                .map_err(|why| format!("line {line}, {n}: {why}"))?;
            cells.push(Cell {
                line: *line,
                column: n.clone(),
                typed: typed.to_string(),
                said: if value == a.nominal.clone().unwrap_or_default() {
                    a.said.clone()
                } else {
                    format!("{}; written as {value} {port_unit}", a.said)
                },
            });
            r.push(value);
        }
        if !own {
            r.push(origin.trim().to_string());
        }
        out.push(r);
    }
    Ok(Read {
        csv: csv::write(&written, &out),
        cases: out.len(),
        cells,
    })
}
