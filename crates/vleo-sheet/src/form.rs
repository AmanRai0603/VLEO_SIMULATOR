//! The declaration form — what a sheet still needs, and why each answer matters.
//!
//! Nine questions, in the order a person is asked them. `xtask declare` prints
//! them for a terminal and the daemon serves them to the web face, so a person
//! filling a row in a browser is asked exactly what a person at a terminal is
//! asked. A second copy of this list in JavaScript would drift the first week
//! somebody added a field, and the drift would be invisible: both forms would
//! look complete.
//!
//! It lives here because this crate is the single implementation of what a
//! sheet *means*. `unfilled` is the function the generators refuse on, so the
//! form and the generator cannot disagree about what a finished sheet is.

use crate::model::Sheet;

/// Which of the nine required fields are still blank.
///
/// `docs` refuses to emit a scaffold while any of these is open: the signature
/// needs the type and the unit, and a bound with no reason is a guard the next
/// person deletes. So this is the one list, read by the generator and by both
/// forms.
pub fn unfilled(sh: &Sheet) -> Vec<&'static str> {
    let mut missing = Vec::new();
    for (name, v) in [
        ("label", &sh.label),
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
    missing
}

/// One question on the form.
pub struct Ask {
    pub field: &'static str,
    /// The question, as a person is asked it.
    pub ask: &'static str,
    /// What cannot be emitted without it. A field with no stated consequence is
    /// a field somebody fills with anything to make the form go green.
    pub why: &'static str,
    pub open: bool,
}

/// The nine questions, with the current answer's state.
///
/// Errors when a field is blank and does not block generation, which would mean
/// this form and `docs` disagree about what a finished sheet is. Two lists that
/// must agree are two lists that will not, so the disagreement is a hard error
/// rather than a difference nobody notices.
pub fn asks(sh: &Sheet) -> Result<Vec<Ask>, String> {
    const SPEC: &[(&str, &str, &str)] = &[
        (
            "label",
            "what is this row called, in the tree",
            "the page title and every reference to it",
        ),
        (
            "question",
            "what one question does it answer",
            "an equation with no question gets reused for the wrong thing",
        ),
        (
            "expression",
            "what is the relation",
            "the algorithm, and what a reviewer compares against the source",
        ),
        (
            "source",
            "cited where — book, paper, page",
            "this is the claim everything else rests on",
        ),
        (
            "symbol",
            "what is the answer's symbol",
            "the binding name in the generated signature",
        ),
        (
            "type",
            "what quantity is it",
            "the signature; a dimensional error has to fail to compile",
        ),
        (
            "unit",
            "in what unit",
            "the conversion at every face boundary",
        ),
        (
            "reason_lower",
            "why is the lower bound there",
            "a guard whose reason is not written gets deleted by the next person",
        ),
        (
            "reason_upper",
            "why is the upper bound there",
            "the same, at the other end",
        ),
    ];
    let blocking = unfilled(sh);
    let mut out = Vec::new();
    for (field, ask, why) in SPEC {
        let open = blocking.contains(field);
        if !open && value(sh, field).trim().is_empty() {
            return Err(format!(
                "'{field}' is blank and does not block generation — the form and docs \
                 disagree about what a finished sheet is"
            ));
        }
        out.push(Ask {
            field,
            ask,
            why,
            open,
        });
    }
    Ok(out)
}

/// What the sheet currently says for one form field.
pub fn value<'a>(sh: &'a Sheet, field: &str) -> &'a str {
    match field {
        "label" => &sh.label,
        "question" => &sh.question,
        "expression" => &sh.expression,
        "source" => &sh.source,
        "symbol" => &sh.symbol,
        "type" => &sh.ty,
        "unit" => &sh.unit,
        "reason_lower" => &sh.reason_lower,
        "reason_upper" => &sh.reason_upper,
        _ => "",
    }
}

/// A JSON string body, escaped. Smaller to write than to depend on.
fn jq(v: &str) -> String {
    let mut o = String::with_capacity(v.len() + 2);
    o.push('"');
    for c in v.chars() {
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
}

/// The form as JSON, for a face to render.
///
/// `structural` is included so the form can SHOW those fields and refuse to
/// edit them. Changing a parent or an order moves the tree and renumbers its
/// neighbours; that is a developer's act at a terminal, not a text box.
///
/// `sheet_hash` is what a later save sends back to prove which version it
/// started from, so two people editing one row cannot overwrite each other
/// silently.
pub fn json(sh: &Sheet) -> Result<String, String> {
    let asks = asks(sh)?;
    let mut o = String::from("{\n");
    o.push_str(&format!("  \"id\": {},\n", jq(&sh.id)));
    o.push_str(&format!("  \"label\": {},\n", jq(&sh.label)));
    o.push_str(&format!(
        "  \"sheet_hash\": {},\n",
        jq(&crate::short_hex(sh.sheet_hash))
    ));
    o.push_str(&format!("  \"criticality\": {},\n", jq(&sh.criticality)));
    o.push_str("  \"structural\": {\n");
    let st = [
        ("kind", sh.kind.clone()),
        ("subsystem", sh.subsystem.clone()),
        ("parent", sh.parent.clone()),
        ("owner", sh.owner.clone()),
        ("tier", sh.tier.clone()),
        ("state", sh.state.clone()),
        ("layer", sh.layer.to_string()),
        ("order", sh.order.to_string()),
    ];
    for (i, (k, v)) in st.iter().enumerate() {
        o.push_str(&format!(
            "    {}: {}{}\n",
            jq(k),
            jq(v),
            if i + 1 == st.len() { "" } else { "," }
        ));
    }
    o.push_str("  },\n  \"fields\": [\n");
    for (i, a) in asks.iter().enumerate() {
        o.push_str(&format!(
            "    {{\"field\": {}, \"ask\": {}, \"why\": {}, \"value\": {}, \"open\": {}}}{}\n",
            jq(a.field),
            jq(a.ask),
            jq(a.why),
            jq(value(sh, a.field)),
            a.open,
            if i + 1 == asks.len() { "" } else { "," }
        ));
    }
    o.push_str("  ],\n");
    // Neither of these blocks generation, and both decide whether the row
    // answers at all, so they are reported apart from the nine rather than
    // mixed in with them.
    o.push_str(&format!(
        "  \"derivation\": {{\"wanted\": {}, \"present\": {}}},\n",
        !sh.steps.is_empty(),
        !sh.steps.is_empty() && !sh.theory.is_empty()
    ));
    o.push_str(&format!(
        "  \"attribution\": {{\"wanted\": {}, \"who\": {}}},\n",
        !sh.expression.trim().is_empty(),
        jq(&sh.relation_by)
    ));
    o.push_str(&format!(
        "  \"open\": {}\n}}\n",
        asks.iter().filter(|a| a.open).count()
    ));
    Ok(o)
}

/// Where each form field lives in the file: its table, and its key.
///
/// An empty table means a top-level key.
pub fn place(field: &str) -> Option<(&'static str, &'static str)> {
    Some(match field {
        "label" => ("", "label"),
        "question" => ("question", "text"),
        "expression" => ("maths", "expression"),
        "source" => ("maths", "source"),
        "confirmed_by" => ("maths", "confirmed_by"),
        "symbol" => ("output", "symbol"),
        "type" => ("output", "type"),
        "unit" => ("output", "unit"),
        "reason_lower" => ("output", "reason_lower"),
        "reason_upper" => ("output", "reason_upper"),
        _ => return None,
    })
}

/// The fields a face may never write, and why.
///
/// Not a blocklist of names but the reason each one is refused, because a
/// reader who is told "no" and not "why" goes looking for a way round.
pub fn structural(field: &str) -> Option<&'static str> {
    Some(match field {
        "id" | "folder" => "the identifier is the folder and the variable name; renaming it is a move",
        "parent" => "the parent is the tree's shape — moving a row moves everyone who reads it",
        "order" => "order decides position among siblings, and the block may be packed solid; \
                    inserting renumbers its neighbours",
        "layer" => "the layer decides which contract the row sits under",
        "kind" => "kind decides whether the row declares a value or computes one, which changes \
                   what is generated for it",
        "subsystem" | "owner" => "ownership is generated into CODEOWNERS and decides who reviews it",
        "tier" | "state" => "both change what the gate demands of the row",
        _ => return None,
    })
}

/// Replace one field's value in a sheet's text, leaving everything else alone.
///
/// TEXTUAL, never a TOML round-trip. These sheets carry more comment than
/// content and every comment is somebody's reason; a serialiser would silently
/// drop the lot. The same choice `xtask confirm` makes, for the same reason.
///
/// Refuses rather than guesses: an absent key, a duplicated one, or a value
/// written as a multi-line string are all reported instead of being patched
/// approximately. A sheet edited approximately is worse than one not edited.
pub fn set(text: &str, field: &str, value: &str) -> Result<String, String> {
    let Some((table, key)) = place(field) else {
        return Err(format!("'{field}' is not a field this form writes"));
    };
    // The window this key must be found in: from its table header to the next
    // one. Without it, `source` under [maths] and a `source` under some other
    // table are the same search.
    let (from, to) = if table.is_empty() {
        (0, text.find("\n[").unwrap_or(text.len()))
    } else {
        let header = format!("\n[{table}]\n");
        let Some(h) = text.find(&header) else {
            return Err(format!("this sheet has no [{table}] table to write {key} into"));
        };
        let start = h + header.len();
        let end = text[start..]
            .find("\n[")
            .map(|i| start + i)
            .unwrap_or(text.len());
        (start, end)
    };
    let window = &text[from..to];

    // Every line in the window that assigns this key.
    //
    // A commented-out assignment needs no guard: the key extracted from
    // `# unit = "Foot"` is `# unit`, which is not `unit`, so it never matches. A
    // check for it was here and nothing could reach it — removed rather than
    // left looking load-bearing.
    //
    // A key-looking line INSIDE a multi-line body does match, and that is what
    // the duplicate refusal below is really protecting: it counts two and
    // refuses, rather than replacing a line in somebody's prose.
    let mut hits = Vec::new();
    let mut at = from;
    for line in window.split_inclusive('\n') {
        let t = line.trim_start();
        if let Some(eq) = t.find('=') {
            if t[..eq].trim() == key {
                hits.push((at, at + line.len(), line));
            }
        }
        at += line.len();
    }
    if hits.is_empty() {
        return Err(format!(
            "no `{key} =` in {} — this form replaces a value that is already there, it does not \
             decide where a new key belongs",
            if table.is_empty() { "the sheet's head" } else { table }
        ));
    }
    if hits.len() > 1 {
        return Err(format!(
            "`{key}` is assigned {} times in [{table}] — refusing to guess which one is meant",
            hits.len()
        ));
    }
    let (s0, s1, line) = hits[0];
    // A multi-line string is a body somebody laid out on purpose. Replacing it
    // by line would truncate it at the first newline and leave the rest as
    // stray TOML, so it is refused by name.
    let rhs = line.split_once('=').map(|x| x.1.trim()).unwrap_or("");
    if rhs.starts_with("\"\"\"") && !(rhs.len() > 5 && rhs.ends_with("\"\"\"")) {
        return Err(format!(
            "`{key}` is a multi-line string. Editing one through this form would truncate it; \
             edit the sheet directly"
        ));
    }
    let mut o = String::with_capacity(text.len() + value.len());
    o.push_str(&text[..s0]);
    o.push_str(&format!("{key} = {}\n", toml_quote(value)));
    o.push_str(&text[s1..]);
    Ok(o)
}

/// A TOML string literal. Prefers a basic string, falls back to a literal one
/// where the value has backslashes worth keeping as typed.
fn toml_quote(v: &str) -> String {
    if v.contains('\n') {
        // A form field that has become multi-line is written as one, so the
        // file stays parseable rather than losing the tail.
        return format!("\"\"\"\n{}\"\"\"", if v.ends_with('\n') { v.to_string() } else { format!("{v}\n") });
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
