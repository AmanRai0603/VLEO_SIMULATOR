//! The declaration form: the questions, and a sheet's answers as the faces read them.

use super::*;

/// One question on the form, with the current answer's state.
pub struct Ask {
    pub field: &'static str,
    pub ask: &'static str,
    pub why: &'static str,
    pub shape: &'static Shape,
    pub group: &'static str,
    /// Whether a blank here blocks generation AND it is blank.
    pub open: bool,
    /// Whether this row has the key at all. A field the sheet does not carry
    /// and the form may not add is shown as unavailable rather than as an empty
    /// box that refuses on save: `sense` belongs to a requirement and
    /// `declared_value` to a declared row, and offering either on a row that is
    /// neither is a question with no right answer.
    pub available: bool,
}

/// The questions, with the current answer's state.
pub fn asks(sh: &Sheet) -> Vec<Ask> {
    let blocking = unfilled(sh);
    let text = std::fs::read_to_string(sh.dir.join("node.toml")).unwrap_or_default();
    FIELDS
        .iter()
        .filter(|f| f.asked)
        .map(|f| Ask {
            field: f.field,
            ask: f.ask,
            why: f.why,
            shape: &f.shape,
            group: f.group,
            open: blocking.contains(&f.field),
            available: f.insert || has_key(&text, f.table, f.key).is_some(),
        })
        .collect()
}

/// What the sheet currently says for one form field.
///
/// A `String` rather than a borrow, because a bound is an `f64` on the sheet and
/// there is nothing to borrow. Written with `{:?}` so an integral value keeps
/// its decimal point and a save that does not change it leaves no diff.
pub fn value(sh: &Sheet, field: &str) -> String {
    match field {
        "label" => sh.label.clone(),
        "question" => sh.question.clone(),
        "note" => sh.note.clone(),
        "expression" => sh.expression.clone(),
        "source" => sh.source.clone(),
        "confirmed_by" => sh.relation_by.clone(),
        "symbol" => sh.symbol.clone(),
        "type" => sh.ty.clone(),
        "unit" => sh.unit.clone(),
        "lower" => format!("{:?}", sh.lower),
        "upper" => format!("{:?}", sh.upper),
        "reason_lower" => sh.reason_lower.clone(),
        "reason_upper" => sh.reason_upper.clone(),
        "sense" => sh.sense.clone(),
        "declared_value" => sh.value.map(|v| format!("{v:?}")).unwrap_or_default(),
        "theory_why" => sh.theory.why.clone(),
        "theory_reading" => sh.theory.reading.clone(),
        "explain_simply" => sh.explain.simply.clone(),
        "explain_breaks" => sh.explain.breaks.clone(),
        "explain_wrong" => sh.explain.wrong.clone(),
        "explain_by" => sh.explain.by.clone(),
        "method_text" => sh.method.text.clone(),
        "method_by" => sh.method.by.clone(),
        "author_name" => sh.author.name.clone(),
        "author_language" => sh.author.language.clone(),
        "author_entry" => sh.author.entry.clone(),
        "author_code" => sh.author.code.clone(),
        "author_test_code" => sh.author.test_code.clone(),
        "author_how_run" => sh.author.how_run.clone(),
        _ => String::new(),
    }
}

/// A JSON string body, escaped. Smaller to write than to depend on.
pub(crate) fn jq(v: &str) -> String {
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
pub fn json(sh: &Sheet) -> Result<String, Error> {
    let asks = asks(sh);
    let mut o = String::from("{\n");
    o.push_str(&format!("  \"id\": {},\n", jq(&sh.id)));
    o.push_str(&format!("  \"label\": {},\n", jq(&sh.label)));
    o.push_str(&format!(
        "  \"sheet_hash\": {},\n",
        jq(&crate::short_hex(sh.sheet_hash))
    ));
    // What a save must send back. See `file_hash`: the sheet hash is not it.
    o.push_str(&format!(
        "  \"file_hash\": {},\n",
        jq(&std::fs::read_to_string(sh.dir.join("node.toml"))
            .map(|t| file_hash(&t))
            .unwrap_or_default())
    ));
    o.push_str(&format!("  \"criticality\": {},\n", jq(&sh.criticality)));
    // Who a save will be attributed to. Shown, never typed: see `git_identity`.
    match git_identity(&sh.dir) {
        Ok(w) => o.push_str(&format!("  \"identity\": {},\n", jq(&w))),
        Err(e) => o.push_str(&format!(
            "  \"identity\": null,\n  \"identity_why\": {},\n",
            jq(e.message())
        )),
    }
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
    // AND THE REASON EACH IS LOCKED, from `structural` rather than from a copy
    // in the face. A reader told "no" and not "why" goes looking for a way
    // round, and the face's own copy of these sentences had already drifted from
    // the server's — it still said `state` "changes what the gate demands",
    // which is not why a state may not be typed into a box.
    o.push_str("  },\n  \"structural_why\": {\n");
    for (i, (k, _)) in st.iter().enumerate() {
        o.push_str(&format!(
            "    {}: {}{}\n",
            jq(k),
            jq(structural(k).unwrap_or("structural")),
            if i + 1 == st.len() { "" } else { "," }
        ));
    }
    o.push_str("  },\n");
    // THE CLOSED SETS, so the face can offer them instead of a text box. Sent
    // rather than duplicated in JavaScript: a second copy would drift the first
    // time a quantity was added, and the drift would be a field the form will
    // not let anybody choose.
    o.push_str("  \"choices\": {\n    \"type\": [");
    for (i, q) in vleo_units::QUANTITIES.iter().enumerate() {
        o.push_str(&format!("{}{}", if i > 0 { ", " } else { "" }, jq(q)));
    }
    o.push_str("],\n    \"unit\": [");
    for (i, u) in crate::unit_names().iter().enumerate() {
        o.push_str(&format!("{}{}", if i > 0 { ", " } else { "" }, jq(u)));
    }
    o.push_str("]\n  },\n");
    // The groups, in the order the form asks them, so a face lays the questions
    // out in the few decisions they belong to rather than as one long column.
    o.push_str("  \"groups\": [");
    for (i, g) in groups().iter().enumerate() {
        o.push_str(&format!("{}{}", if i > 0 { ", " } else { "" }, jq(g)));
    }
    o.push_str("],\n  \"fields\": [\n");
    for (i, a) in asks.iter().enumerate() {
        o.push_str(&format!(
            "    {{\"field\": {}, \"ask\": {}, \"why\": {}, \"value\": {}, \"open\": {}, \
             \"group\": {}, \"shape\": {}, \"options\": [{}], \"available\": {}}}{}\n",
            jq(a.field),
            jq(a.ask),
            jq(a.why),
            jq(&value(sh, a.field)),
            a.open,
            jq(a.group),
            jq(a.shape.name()),
            a.shape
                .options()
                .iter()
                .map(|o| jq(o))
                .collect::<Vec<_>>()
                .join(", "),
            a.available,
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
    // The repeated blocks, with their current contents. Read from the loaded
    // sheet rather than from the file: the parser is the authority on what a
    // block SAYS, and it is only writing that has to be textual.
    o.push_str("  \"arrays\": [\n");
    for (i, a) in ARRAYS.iter().enumerate() {
        o.push_str(&format!(
            "    {{\"name\": {}, \"label\": {}, \"why\": {}, \"mode\": {},\n",
            jq(a.name),
            jq(a.label),
            jq(a.why),
            jq(if a.blocks == Blocks::EndOnly {
                "end"
            } else {
                "free"
            })
        ));
        o.push_str("     \"columns\": [");
        for (j, c) in a.columns.iter().enumerate() {
            o.push_str(&format!(
                "{}{{\"key\": {}, \"shape\": {}, \"ask\": {}, \"required\": {}, \"managed\": {}, \"options\": [{}]}}",
                if j > 0 { ", " } else { "" },
                jq(c.key),
                jq(c.shape.name()),
                jq(c.ask),
                c.required,
                c.managed,
                c.shape
                    .options()
                    .iter()
                    .map(|o| jq(o))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        o.push_str("],\n     \"rows\": [");
        let rows = array_rows(sh, a);
        for (j, r) in rows.iter().enumerate() {
            o.push_str(&format!(
                "{}{{{}}}",
                if j > 0 { ", " } else { "" },
                r.iter()
                    .map(|(k, v)| format!("{}: {}", jq(k), jq(v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        o.push_str(&format!(
            "]}}{}\n",
            if i + 1 == ARRAYS.len() { "" } else { "," }
        ));
    }
    o.push_str("  ],\n");
    // How the answer is drawn, as one state rather than three fields: the kind
    // decides which of the others exist.
    let (vk, vo, vp) = match &sh.view {
        crate::model::View::Number => ("number", String::new(), 0),
        crate::model::View::Line { over, points } => ("line", over.clone(), *points),
        crate::model::View::Bar { y } => ("bar", y.clone(), 0),
        crate::model::View::Heatmap { over_x, .. } => ("heatmap", over_x.clone(), 0),
    };
    o.push_str(&format!(
        "  \"view\": {{\"kind\": {}, \"over\": {}, \"points\": {}, \"kinds\": [{}]}},\n",
        jq(vk),
        jq(&vo),
        vp,
        VIEW_KINDS
            .iter()
            .map(|k| jq(k))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    // Whether this row can be published, and if not, every reason. A state is
    // not a field: see `publish`.
    let why = unpublishable(sh);
    o.push_str(&format!(
        "  \"publish\": {{\"state\": {}, \"possible\": {}, \"why\": [{}]}},\n",
        jq(&sh.state),
        why.is_empty(),
        why.iter().map(|w| jq(w)).collect::<Vec<_>>().join(", ")
    ));
    o.push_str(&format!(
        "  \"open\": {}\n}}\n",
        asks.iter().filter(|a| a.open).count()
    ));
    Ok(o)
}

/// One repeated block's current contents, as key/value pairs per block.
pub(crate) fn array_rows(sh: &Sheet, a: &Array) -> Vec<Vec<(&'static str, String)>> {
    match a.path {
        "input" => sh
            .inputs
            .iter()
            .map(|i| {
                vec![
                    ("binding", i.binding.clone()),
                    ("var", i.var.clone()),
                    ("type", i.ty.clone()),
                ]
            })
            .collect(),
        "algorithm.step" => sh
            .steps
            .iter()
            .map(|s| {
                vec![
                    ("number", s.number.to_string()),
                    ("text", s.text.clone()),
                    ("binds", s.binds.clone()),
                    ("type", s.ty.clone()),
                ]
            })
            .collect(),
        "theory.step" => sh
            .theory
            .steps
            .iter()
            .map(|s| vec![("text", s.text.clone()), ("math", s.math.clone())])
            .collect(),
        "assumption" => sh
            .assumptions
            .iter()
            .map(|x| {
                vec![
                    ("text", x.text.clone()),
                    ("fails_when", x.fails_when.clone()),
                ]
            })
            .collect(),
        "risk" => sh
            .risks
            .iter()
            .map(|r| {
                vec![
                    ("id", r.id.clone()),
                    ("title", r.title.clone()),
                    ("level", r.level.clone()),
                    ("owner", r.owner.clone()),
                    ("since", r.since.clone()),
                    ("why", r.why.clone()),
                ]
            })
            .collect(),
        "case" => sh
            .cases
            .iter()
            .map(|c| {
                let num = |v: f64| {
                    if v == 0.0 {
                        String::new()
                    } else {
                        format!("{v:?}")
                    }
                };
                let inputs = c
                    .inputs
                    .iter()
                    .map(|(k, v)| format!("{k} = {v:?}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                vec![
                    ("label", c.label.clone()),
                    ("refuse", if c.refuses() { "yes" } else { "no" }.to_string()),
                    (
                        "expect",
                        c.expect.map(|v| format!("{v:?}")).unwrap_or_default(),
                    ),
                    ("tolerance", num(c.tolerance)),
                    ("inputs", format!("{{ {inputs} }}")),
                ]
            })
            .collect(),
        "flight" => sh
            .flight
            .iter()
            .map(|f| {
                vec![
                    ("name", f.name.clone()),
                    ("language", f.language.clone()),
                    ("purpose", f.purpose.clone()),
                    ("code", f.code.clone()),
                    ("test_code", f.test_code.clone()),
                    ("test_result", f.test_result.clone()),
                ]
            })
            .collect(),
        _ => Vec::new(),
    }
}
