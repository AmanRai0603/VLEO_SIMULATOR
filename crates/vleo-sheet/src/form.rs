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
