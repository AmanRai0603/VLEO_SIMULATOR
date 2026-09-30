//! Writing one value into a sheet's text: finding its table and key, keeping
//! every comment, and writing the value in the shape its field takes.

use super::*;

/// The key a line assigns, and what it assigns, or nothing.
///
/// A key is a bare name. A line whose left side has a space, a tab, a hash or a
/// bracket in it is a comment, a table header or prose — never an assignment —
/// which is why `# unit = "Foot"` needs no special case anywhere: the name it
/// yields is `# unit`, and that is not a key.
pub(crate) fn key_and_rhs(line: &str) -> Option<(&str, &str)> {
    let (lhs, rhs) = line.trim_start().split_once('=')?;
    let name = lhs.trim();
    if name.is_empty() || name.contains([' ', '\t', '#', '[']) {
        return None;
    }
    Some((name, rhs.trim()))
}

/// Whether this right-hand side opens a multi-line string that later lines
/// continue. `"""x"""` on one line does not.
pub(crate) fn opens_prose(rhs: &str) -> bool {
    rhs.starts_with("\"\"\"") && !(rhs.len() >= 6 && rhs.ends_with("\"\"\""))
}

/// The byte range of one table's body: from just after its header line to the
/// start of the next table header. An empty name means the sheet's head.
///
/// BY LINE, NOT BY SEARCHING FOR `"\n["`. That search was wrong in two ways and
/// one of them showed. A table with an EMPTY body is followed immediately by the
/// next header, which then sits at offset 0 of the remaining slice with no
/// newline in front of it — so the search skipped it and the window swallowed
/// the whole of the following table. A `[theory]` created and not yet written
/// into is exactly that case, and a theory step added to such a sheet landed
/// after `[output]` instead of under the table it belongs to. The same search
/// would also have matched a line beginning `[` inside somebody's paragraph.
///
/// `scan` already knows which lines are prose and which are not, so the answer
/// is to ask it rather than to search the bytes.
pub(crate) fn window(text: &str, table: &str) -> Option<(usize, usize)> {
    let header = (!table.is_empty()).then(|| format!("[{table}]"));
    let mut start: Option<usize> = None;
    for r in scan(text) {
        if r.prose {
            continue;
        }
        let line = text[r.start..r.end].trim();
        if !line.starts_with('[') {
            continue;
        }
        match (&header, start) {
            // The sheet's head runs to the first table header of any kind.
            (None, _) => return Some((0, r.start)),
            (Some(h), None) if line == h => start = Some(r.end),
            (Some(_), Some(s)) => return Some((s, r.start)),
            _ => {}
        }
    }
    match (header, start) {
        (None, _) => Some((0, text.len())),
        (Some(_), Some(s)) => Some((s, text.len())),
        (Some(_), None) => None,
    }
}

/// Every assignment of `key` in `table`, as a byte range covering the whole
/// value — a multi-line block included — and the trailing comment to keep.
///
/// A LINE THAT LOOKS LIKE AN ASSIGNMENT BUT SITS INSIDE A MULTI-LINE STRING IS
/// PROSE. The first version of this counted those as assignments and refused the
/// edit as ambiguous, so a row whose reason happened to contain `unit = ...`
/// could not have its unit changed at all. The blocks are tracked here and their
/// contents skipped, which is both correct and fewer refusals.
///
/// A commented-out assignment needs no special case: the key extracted from
/// `# unit = "Foot"` is `# unit`, which is not `unit`.
pub(crate) fn assignments(text: &str, table: &str, key: &str) -> Vec<(usize, usize, String)> {
    let Some((from, to)) = window(text, table) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    // Where the open multi-line block's assignment started, and whether it is
    // the key being looked for. Tracked for EVERY key, not only this one: a
    // block opened by another key is where the prose that must be skipped is.
    let mut open: Option<(usize, bool)> = None;
    let mut at = from;
    for line in text[from..to].split_inclusive('\n') {
        let end = at + line.len();
        match open {
            Some((start, wanted)) => {
                if let Some(i) = line.find("\"\"\"") {
                    if wanted {
                        out.push((start, end, trailing_comment(&line[i + 3..])));
                    }
                    open = None;
                }
            }
            None => {
                if let Some((name, rhs)) = key_and_rhs(line) {
                    if opens_prose(rhs) {
                        open = Some((at, name == key));
                    } else if name == key {
                        out.push((at, end, trailing_comment(rhs)));
                    }
                }
            }
        }
        at = end;
    }
    out
}

/// The comment after a value, so replacing the value keeps it.
///
/// 27 of the fields this form writes carry one, and they are not decoration:
/// `# REQUIRED — an assistant may never supply mathematics` sits on the relation of
/// the rows where that matters most. A save that dropped it would remove the
/// instruction from the one place the next person reads.
///
/// Takes the text after the value's opening delimiter and gives back the comment
/// with its leading spacing, or nothing.
pub(crate) fn trailing_comment(rhs: &str) -> String {
    // Where the value ends. A quoted string ends at its unescaped closing
    // quote; a bare number ends at the first space or hash.
    let bytes = rhs.as_bytes();
    let rest = match bytes.first() {
        Some(b'"') | Some(b'\'') => {
            let q = bytes[0];
            let mut i = 1;
            let mut esc = false;
            let mut end = None;
            while i < bytes.len() {
                if esc {
                    esc = false;
                } else if bytes[i] == b'\\' && q == b'"' {
                    esc = true;
                } else if bytes[i] == q {
                    end = Some(i + 1);
                    break;
                }
                i += 1;
            }
            match end {
                Some(e) => &rhs[e..],
                // An unterminated string — say nothing rather than guess.
                None => return String::new(),
            }
        }
        // Already past the value: the caller handed over what follows a closing
        // `"""`, or this is a bare value.
        _ => {
            let cut = rhs
                .find(|c: char| c.is_whitespace() || c == '#')
                .unwrap_or(rhs.len());
            &rhs[cut..]
        }
    };
    let t = rest.trim_end_matches('\n').trim_end();
    if t.trim_start().starts_with('#') {
        // The spacing is somebody's alignment; keep it, but never less than one
        // space or the comment would run into the value.
        let lead = &t[..t.len() - t.trim_start().len()];
        format!(
            "{}{}",
            if lead.is_empty() { "   " } else { lead },
            t.trim_start()
        )
    } else {
        String::new()
    }
}

/// Whether the sheet carries this key at all, and where.
pub(crate) fn has_key(text: &str, table: &str, key: &str) -> Option<(usize, usize)> {
    assignments(text, table, key)
        .first()
        .map(|(a, b, _)| (*a, *b))
}

/// Add a table the sheet has not got, in the place the authored sheets put it.
///
/// Only `[theory]` and `[explain]` are ever missing — `[question]`, `[maths]`,
/// `[output]` and `[view]` are on all 1396 rows — so this creates those two and
/// refuses the rest. A missing `[output]` is a malformed sheet and inventing it
/// here would hide that.
pub(crate) fn ensure_table(text: &str, table: &str) -> Result<String, String> {
    if window(text, table).is_some() {
        return Ok(text.to_string());
    }
    // THE ROW SAID SIMPLY goes straight after its question, which is where a
    // reader meets it on the page (docs/EXPLAINING.md, E2). Without this, every
    // row but the one that already had the table refused its own first
    // plain-words answer — which is every row a form was ever sent for.
    if table == "explain" {
        let (_, end) = window(text, "question").ok_or_else(|| {
            "this sheet has no [question] table to put [explain] after".to_string()
        })?;
        let mut o = String::with_capacity(text.len() + 64);
        o.push_str(text[..end].trim_end_matches('\n'));
        o.push_str(
            "\n\n# SAID SIMPLY, AND WHERE THAT BREAKS — docs/EXPLAINING.md. Prose, outside the\n\
             # sheet hash.\n[explain]\n\n",
        );
        o.push_str(text[end..].trim_start_matches('\n'));
        return Ok(o);
    }
    // THE METHOD AND THE AUTHOR'S CODE go after the relation and its
    // derivation, where a reader who has just read the expression meets the
    // same thing again as something that runs.
    if table == "method" || table == "author" {
        // After the derivation — its last step, or its table — or else the
        // relation; and the author's code after the method when there is one.
        let after_derivation = array_blocks(text, "theory.step")
            .last()
            .map(|(_, e)| *e)
            .or_else(|| window(text, "theory").map(|(_, e)| e))
            .or_else(|| window(text, "maths").map(|(_, e)| e));
        let end = if table == "author" {
            window(text, "method").map(|(_, e)| e).or(after_derivation)
        } else {
            after_derivation
        }
        .ok_or_else(|| format!("this sheet has no [maths] table to put [{table}] after"))?;
        let head = if table == "method" {
            "# THE METHOD — the relation in the method language (docs/PSEUDOCODE.md). In the\n\
             # sheet hash: the generated code is translated from it.\n[method]\n\n"
        } else {
            "# THE AUTHOR'S OWN CODE, which produced the cases below, and the script that ran\n\
             # it. Evidence, outside the sheet hash.\n[author]\n\n"
        };
        let mut o = String::with_capacity(text.len() + 128);
        o.push_str(text[..end].trim_end_matches('\n'));
        o.push_str("\n\n");
        o.push_str(head);
        o.push_str(text[end..].trim_start_matches('\n'));
        return Ok(o);
    }
    if table != "theory" {
        return Err(format!(
            "this sheet has no [{table}] table, which every sheet should have. That is a \
             malformed sheet and not something a form should paper over — edit it directly"
        ));
    }
    // Immediately after [maths], which is where every authored sheet has it and
    // which is before the [[theory.step]] blocks a later edit may add.
    let (_, end) = window(text, "maths")
        .ok_or_else(|| "this sheet has no [maths] table to put [theory] after".to_string())?;
    let mut o = String::with_capacity(text.len() + 64);
    o.push_str(text[..end].trim_end_matches('\n'));
    o.push_str(
        "\n\n# WHERE THIS RELATION CAME FROM. Prose, outside the sheet hash: correcting a\n\
         # sentence here does not invalidate a generated artefact.\n[theory]\n\n",
    );
    o.push_str(text[end..].trim_start_matches('\n'));
    Ok(o)
}

/// Replace one field's value in a sheet's text, leaving everything else alone.
///
/// TEXTUAL, never a TOML round-trip. These sheets carry more comment than
/// content and every comment is somebody's reason; a serialiser would silently
/// drop the lot. The same choice `xtask confirm` makes, for the same reason.
///
/// Refuses rather than guesses: a duplicated key, or a multi-line block under a
/// field whose shape is a number, are reported instead of being patched
/// approximately. A sheet edited approximately is worse than one not edited.
///
/// An ABSENT key is added where the field says it may be — most sheets have no
/// `note` and no `[theory]`, so a form that could only replace could never write
/// either — and refused where it may not.
pub fn set(text: &str, field: &str, value: &str) -> Result<String, String> {
    let Some(f) = self::field(field) else {
        return Err(format!("'{field}' is not a field this form writes"));
    };
    let value = normalise(field, value)?;
    let text = if has_key(text, f.table, f.key).is_none() && f.insert {
        ensure_table(text, f.table)?
    } else {
        text.to_string()
    };
    let text = text.as_str();

    let hits = assignments(text, f.table, f.key);
    if hits.len() > 1 {
        return Err(format!(
            "`{}` is assigned {} times in {} — refusing to guess which one is meant",
            f.key,
            hits.len(),
            if f.table.is_empty() {
                "the sheet's head".to_string()
            } else {
                format!("[{}]", f.table)
            }
        ));
    }
    let Some((s0, s1, comment)) = hits.into_iter().next() else {
        if !f.insert {
            return Err(format!(
                "no `{} =` in {} — this row has not got that field, and this form does not \
                 decide where a new one belongs. {}",
                f.key,
                if f.table.is_empty() {
                    "the sheet's head".to_string()
                } else {
                    format!("[{}]", f.table)
                },
                match field {
                    "sense" =>
                        "A sense belongs to a requirement; the gate puts one on every \
                                row of that kind, so a row without one is not one.",
                    "declared_value" =>
                        "A declared number belongs to a row that declares one \
                                         rather than computing it.",
                    _ => "Edit the sheet directly.",
                }
            ));
        }
        // Insertable and absent: directly under the table header, which is
        // where `xtask confirm` puts the one key it adds and the only place in
        // a table that is unambiguous. The method and the author's code are
        // the exception: the form writes their keys one after another into a
        // table it made, and under the header would write them backwards.
        let (from, to) = window(text, f.table).ok_or_else(|| {
            format!(
                "this sheet has no [{}] table to write {} into",
                f.table, f.key
            )
        })?;
        let from = if matches!(f.table, "method" | "author") {
            // After the table's last value — a multi-line one included — and
            // before any comment that introduces the next table.
            let mut last = from;
            for r in scan(text) {
                if r.start < from || r.start >= to {
                    continue;
                }
                let line = text[r.start..r.end].trim();
                if r.prose || (!line.is_empty() && !line.starts_with('#')) {
                    last = r.end;
                }
            }
            last
        } else {
            from
        };
        let mut o = String::with_capacity(text.len() + value.len() + 16);
        o.push_str(&text[..from]);
        o.push_str(&format!("{} = {}\n", f.key, written(&f.shape, &value)));
        o.push_str(&text[from..]);
        return Ok(o);
    };
    // A multi-line block under a shape that cannot be prose is a sheet saying
    // something this form has misunderstood. Refused by name rather than
    // flattened into one line.
    if !f.shape.may_be_prose() && text[s0..s1].trim_end().lines().count() > 1 {
        return Err(format!(
            "`{}` is written as a multi-line block, which a {} cannot be. Nothing was \
             written; edit the sheet directly",
            f.key,
            f.shape.name()
        ));
    }
    let mut o = String::with_capacity(text.len() + value.len());
    o.push_str(&text[..s0]);
    // The indentation the assignment had. Every sheet writes these flush left,
    // but taking it from the line rather than assuming it means a hand-indented
    // sheet is not straightened out behind its author's back.
    let indent: String = text[s0..s1]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect();
    o.push_str(&format!(
        "{indent}{} = {}{comment}\n",
        f.key,
        written(&f.shape, &value)
    ));
    o.push_str(&text[s1..]);
    Ok(o)
}

/// The value as it goes into the file: quoted, or bare for a number.
pub(crate) fn written(shape: &Shape, value: &str) -> String {
    if shape.bare() {
        value.to_string()
    } else {
        toml_quote(value)
    }
}

/// A TOML string literal. Prefers a basic string, falls back to a literal one
/// where the value has backslashes worth keeping as typed.
pub(crate) fn toml_quote(v: &str) -> String {
    if v.contains('\n') {
        // A form field that has become multi-line is written as one, so the
        // file stays parseable rather than losing the tail. A basic multi-line
        // string reads a backslash as an escape and ends at the first `"""`,
        // so both are escaped: a formula written `\alpha`, or a line of C
        // with `"\n"` in it, comes back exactly as it was typed.
        let mut body = String::with_capacity(v.len() + 8);
        for c in v.chars() {
            match c {
                '\\' => body.push_str("\\\\"),
                '\n' | '\t' => body.push(c),
                '\r' => {}
                c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                    body.push_str(&format!("\\u{:04x}", c as u32))
                }
                c => body.push(c),
            }
        }
        let body = body.replace("\"\"\"", "\"\"\\\"");
        return format!(
            "\"\"\"\n{}\"\"\"",
            if body.ends_with('\n') {
                body
            } else {
                format!("{body}\n")
            }
        );
    }
    let mut o = String::from("\"");
    for c in v.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// The value as it must be written into the file, or why it cannot be.
///
/// THE GATE DOES NOT DO THIS. It asks whether a field is blank, and a sheet's
/// `[output] type` is emitted verbatim into the generated signature — so a type
/// that is not a real quantity produces `Result<Nonsense, Fault>`, which does
/// not compile. A face with a text box could write that, regenerate, pass the
/// gate and report success, leaving the repository not building. The gate cannot
/// catch it because the gate never compiles anything.
///
/// It also NORMALISES, which is why it gives back the value rather than a
/// verdict. A bound typed as `40` is written `40.0`: TOML reads the first as an
/// integer, and while the loader takes either, a form that turned every float in
/// the tree into an integer on the way past would be rewriting 1396 sheets for
/// nothing. A number that does not parse is refused here rather than written and
/// silently read back as zero.
pub fn normalise(field: &str, value: &str) -> Result<String, String> {
    let Some(f) = self::field(field) else {
        return Err(format!("'{field}' is not a field this form writes"));
    };
    let v = value.trim();
    match f.shape {
        Shape::Number => {
            let n: f64 = v.parse().map_err(|_| {
                format!(
                    "'{v}' is not a number. It is written into the sheet unquoted and into the \
                     generated guard as an f64; anything else would be read back as zero"
                )
            })?;
            if !n.is_finite() {
                return Err(format!(
                    "'{v}' is not finite. A bound that is not a number cannot guard anything"
                ));
            }
            Ok(format!("{n:?}"))
        }
        Shape::Count => v
            .parse::<u32>()
            .map(|n| n.to_string())
            .map_err(|_| format!("'{v}' is not a whole number of at least zero")),
        Shape::Choice(options) => {
            if options.contains(&v) {
                Ok(v.to_string())
            } else {
                Err(format!(
                    "'{v}' is not one of: {}. This is a closed set, not a label",
                    options.join(", ")
                ))
            }
        }
        Shape::Quantity => {
            if crate::is_quantity_name(v) {
                Ok(v.to_string())
            } else {
                Err(format!(
                    "'{v}' is not a quantity this system has. The type is written straight into \
                     the generated signature, so one that does not exist stops the tree \
                     compiling. One of: {}",
                    vleo_units::QUANTITIES.join(", ")
                ))
            }
        }
        Shape::UnitName => {
            if crate::unit_exists(v) {
                Ok(v.to_string())
            } else {
                Err(format!(
                    "'{v}' is not a unit this system knows, so nothing could convert it at a \
                     face boundary. See vleo_units::Unit for the ones that exist."
                ))
            }
        }
        // A row id is checked against the tree by the gate, which is the only
        // place the whole graph is visible. Refusing the obviously impossible
        // here saves a write and a rollback for a typo with a space in it.
        Shape::RowId => {
            if v.is_empty() || v.contains(char::is_whitespace) {
                Err(format!(
                    "'{v}' is not a row id — an id is one word, and whether it resolves is the \
                     gate's question"
                ))
            } else {
                Ok(v.to_string())
            }
        }
        Shape::Line => {
            if v.contains('\n') {
                Err(format!(
                    "'{field}' is one line. What was sent has {} of them; if the answer needs a \
                     paragraph it belongs in a field that holds one",
                    v.lines().count()
                ))
            } else {
                Ok(v.to_string())
            }
        }
        // Prose keeps its internal newlines and loses only trailing blank ones,
        // because the layout is somebody's.
        Shape::Prose => Ok(value.trim_end().trim_start_matches('\n').to_string()),
        // Code keeps even the first line's indentation.
        Shape::Code => Ok(code_text(value)),
        Shape::Inputs => inputs_text(value),
    }
}

/// Code as it is kept: every line as typed, blank lines at either end gone,
/// line endings made one kind.
pub(crate) fn code_text(value: &str) -> String {
    let v = value.replace("\r\n", "\n");
    let v = v.trim_end();
    let first = v
        .char_indices()
        .find(|(_, c)| *c != '\n')
        .map(|(i, _)| i)
        .unwrap_or(v.len());
    // Back to the start of the first non-blank line, keeping its indent.
    let start = v[..first].rfind('\n').map(|i| i + 1).unwrap_or(0);
    v[start..].to_string()
}

/// A case's inputs as the inline table the sheet holds: `{ h = 250000.0 }`,
/// each value a number, the names in order.
pub(crate) fn inputs_text(value: &str) -> Result<String, String> {
    let v = value.trim();
    let doc: toml::Value = format!("x = {v}").parse().map_err(|_| {
        format!("'{v}' is not a set of inputs — write them as {{ name = number, … }}")
    })?;
    let t = doc
        .get("x")
        .and_then(|x| x.as_table())
        .ok_or_else(|| format!("'{v}' is not a set of inputs"))?;
    let mut parts = Vec::new();
    for (k, x) in t {
        let n = x
            .as_float()
            .or_else(|| x.as_integer().map(|i| i as f64))
            .ok_or_else(|| format!("the input «{k}» is not a number"))?;
        if !n.is_finite() {
            return Err(format!("the input «{k}» is not finite"));
        }
        parts.push(format!("{k} = {n:?}"));
    }
    Ok(format!("{{ {} }}", parts.join(", ")))
}

/// Whether a value is one this field may hold. `normalise` without the value.
pub fn value_allowed(field: &str, value: &str) -> Result<(), String> {
    normalise(field, value).map(|_| ())
}

/// One line of a sheet, and whether it is inside a multi-line string.
///
/// EVERYTHING TEXTUAL HERE NEEDS THIS. A line beginning `[` inside somebody's
/// paragraph is not a table header and a line reading `x = 1` inside one is not
/// an assignment; treating either as what it looks like edits a reason and
/// leaves the field alone.
pub(crate) struct Row {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) prose: bool,
}

pub(crate) fn scan(text: &str) -> Vec<Row> {
    let mut out = Vec::new();
    let mut at = 0;
    let mut open = false;
    for line in text.split_inclusive('\n') {
        let end = at + line.len();
        if open {
            out.push(Row {
                start: at,
                end,
                prose: true,
            });
            if line.contains("\"\"\"") {
                open = false;
            }
        } else {
            out.push(Row {
                start: at,
                end,
                prose: false,
            });
            open = key_and_rhs(line)
                .map(|(_, r)| opens_prose(r))
                .unwrap_or(false);
        }
        at = end;
    }
    out
}
