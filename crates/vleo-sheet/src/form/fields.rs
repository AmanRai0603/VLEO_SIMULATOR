//! Every field a sheet has: its shape, where it lives, whether a person may
//! change it, what it holds now, and what is still unanswered.

use super::*;
pub(crate) use crate::escape::json as jq;

/// What kind of value a field holds.
///
/// A name alone cannot decide three things this does: how the value is written
/// back — `lower = "40"` parses as a string and the sheet loads with a zero, so
/// a number must go in bare — what a face should offer instead of a text box,
/// and what is refused before anything reaches the file.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// One line, written as a quoted string.
    Line,
    /// A paragraph, written as a TOML multi-line string. Replacing one replaces
    /// the whole block: truncating at the first newline and leaving the tail as
    /// stray TOML is the failure this shape exists to stop.
    Prose,
    /// A real number, written unquoted and normalised so it always carries a
    /// decimal point.
    Number,
    /// A whole number, written unquoted.
    Count,
    /// One of a closed set, offered rather than typed.
    Choice(&'static [&'static str]),
    /// A quantity name, against the registry the generated signature is written
    /// from.
    Quantity,
    /// A unit name, against the registry every face boundary converts through.
    UnitName,
    /// A row id. Whether it resolves is a question about the whole tree, so the
    /// gate answers it; this sees one sheet and does not pretend otherwise.
    RowId,
    /// Source code or a method, kept exactly as typed: every line and every
    /// leading space. Written as a multi-line string with its backslashes
    /// escaped, so `printf("\\n")` in somebody's C comes back as they wrote it.
    Code,
    /// One number per input of the node, by binding name, in SI: written as an
    /// inline table, `{ h = 250000.0 }`.
    Inputs,
}

impl Shape {
    /// What a face needs to know to draw the right control.
    pub fn name(&self) -> &'static str {
        match self {
            Shape::Line => "line",
            Shape::Prose => "prose",
            Shape::Number => "number",
            Shape::Count => "count",
            Shape::Choice(_) => "choice",
            Shape::Quantity => "quantity",
            Shape::UnitName => "unit",
            Shape::RowId => "row",
            Shape::Code => "code",
            Shape::Inputs => "inputs",
        }
    }
    /// The closed set, where there is one. `Quantity` and `UnitName` are closed
    /// too, but their sets are the registries and are sent once for the whole
    /// form rather than repeated on every field.
    pub fn options(&self) -> &'static [&'static str] {
        match self {
            Shape::Choice(o) => o,
            _ => &[],
        }
    }
    /// Whether the value goes into the file without quotes.
    pub(crate) fn bare(&self) -> bool {
        matches!(self, Shape::Number | Shape::Count | Shape::Inputs)
    }
    /// Whether an existing multi-line block is something this shape can hold.
    pub(crate) fn may_be_prose(&self) -> bool {
        matches!(self, Shape::Prose | Shape::Code)
    }
}

/// The three ways this tool draws an answer. A closed set, so it is offered
/// rather than typed — and the kind decides which other keys the `[view]` table
/// has, which is why `save_view` writes the table rather than the key.
pub const VIEW_KINDS: &[&str] = &["number", "line", "bar"];

/// Which way a requirement binds. Never defaulted — see the root instructions:
/// *the design sustains Ap 200* and *the design needs Ap 200* are the same
/// number and opposite requirements.
pub const SENSES: &[&str] = &["<=", ">="];

/// The languages an author's own code may be in. A closed set so a reviewer
/// knows what will run it — and `other` for the rest, which is kept and read
/// but never rerun.
pub const LANGUAGES: &[&str] = &[
    "MATLAB", "Octave", "Python", "C", "C++", "Julia", "Fortran", "Rust", "Excel", "other",
];

/// Whether a case must be refused.
pub const YES_NO: &[&str] = &["no", "yes"];

/// One question on the form: where its answer lives in the file, what shape it
/// is, and what cannot be emitted without it.
pub struct Field {
    pub field: &'static str,
    /// The TOML table it lives in. Empty means a top-level key.
    pub table: &'static str,
    pub key: &'static str,
    pub shape: Shape,
    /// Which part of the form it belongs to, so a face can group the questions
    /// into the few things a person is actually deciding rather than listing
    /// them all at once.
    pub group: &'static str,
    /// The question, as a person is asked it.
    pub ask: &'static str,
    /// What cannot be emitted without it. A field with no stated consequence is
    /// a field somebody fills with anything to make the form go green.
    pub why: &'static str,
    /// Whether `docs` refuses to emit a scaffold while it is blank. These are
    /// the nine, and this is the only list of them.
    pub blocks: bool,
    /// Whether the form puts the question to a person. `confirmed_by` is
    /// written and never asked — see `git_identity`.
    pub asked: bool,
    /// Whether the form may add the key when the sheet has not got it. Most
    /// sheets have no `note` and no `[theory]`, so a form that could only
    /// replace could never write either. Where this is false an absent key is
    /// refused rather than invented: `sense` exists on every requirement by the
    /// gate's own check, so its absence means the row is not one.
    pub insert: bool,
}

/// EVERY SCALAR FIELD OF A SHEET THIS FORM WRITES, in the order a person is
/// asked them.
///
/// One table, read by `unfilled`, by both forms, and by `set`. The previous
/// arrangement kept the blocking list and the question list apart and had to
/// raise an error when they disagreed; with one table they cannot, so the error
/// is gone rather than guarded against.
///
/// The repeated blocks — inputs, algorithm steps, theory steps, assumptions —
/// are not here. A block is added and removed as well as edited, which is a
/// different operation with different refusals: see `ARRAYS`.
pub const FIELDS: &[Field] = &[
    Field {
        field: "label",
        table: "",
        key: "label",
        shape: Shape::Line,
        group: "the row",
        ask: "what is this row called, in the tree",
        why: "the page title and every reference to it",
        blocks: true,
        asked: true,
        insert: false,
    },
    Field {
        field: "question",
        table: "question",
        key: "text",
        shape: Shape::Prose,
        group: "the row",
        ask: "what one question does it answer",
        why: "an equation with no question gets reused for the wrong thing",
        blocks: true,
        asked: true,
        insert: false,
    },
    Field {
        field: "note",
        table: "question",
        key: "note",
        shape: Shape::Prose,
        group: "the row",
        ask: "what does a reader need told that the question does not say",
        why: "the sentence under the question on the node's page, and the one place \
              a row can say what it is NOT for",
        blocks: false,
        asked: true,
        insert: true,
    },
    Field {
        field: "explain_simply",
        table: "explain",
        key: "simply",
        shape: Shape::Prose,
        group: "said simply",
        ask: "say it simply: what does this row work out, and why does it matter — in plain \
              words, with no symbol and no word a newcomer would have to look up",
        why: "the first thing on the node's page. A reader who cannot yet read the relation \
              reads this, and a writer who cannot write it has found a gap in their own \
              understanding (docs/EXPLAINING.md, E2)",
        blocks: false,
        asked: true,
        insert: true,
    },
    Field {
        field: "expression",
        table: "maths",
        key: "expression",
        shape: Shape::Line,
        group: "the relation",
        ask: "what is the relation",
        why: "the algorithm, and what a reviewer compares against the source",
        blocks: true,
        asked: true,
        insert: false,
    },
    Field {
        field: "source",
        table: "maths",
        key: "source",
        shape: Shape::Line,
        group: "the relation",
        ask: "cited where — book, paper, page",
        why: "this is the claim everything else rests on",
        blocks: true,
        asked: true,
        insert: false,
    },
    Field {
        field: "confirmed_by",
        table: "maths",
        key: "confirmed_by",
        shape: Shape::Line,
        group: "",
        ask: "",
        why: "",
        blocks: false,
        asked: false,
        insert: true,
    },
    Field {
        field: "symbol",
        table: "output",
        key: "symbol",
        shape: Shape::Line,
        group: "the answer",
        ask: "what is the answer's symbol",
        why: "the binding name in the generated signature",
        blocks: true,
        asked: true,
        insert: false,
    },
    Field {
        field: "type",
        table: "output",
        key: "type",
        shape: Shape::Quantity,
        group: "the answer",
        ask: "what quantity is it",
        why: "the signature; a dimensional error has to fail to compile",
        blocks: true,
        asked: true,
        insert: false,
    },
    Field {
        field: "unit",
        table: "output",
        key: "unit",
        shape: Shape::UnitName,
        group: "the answer",
        ask: "in what unit",
        why: "the conversion at every face boundary",
        blocks: true,
        asked: true,
        insert: false,
    },
    Field {
        field: "lower",
        table: "output",
        key: "lower",
        shape: Shape::Number,
        group: "the answer",
        ask: "what is the lowest value this row may return",
        why: "the guard the generated code refuses below, and half of what the \
              gate calls this row's domain",
        blocks: false,
        asked: true,
        insert: false,
    },
    Field {
        field: "upper",
        table: "output",
        key: "upper",
        shape: Shape::Number,
        group: "the answer",
        ask: "and the highest",
        why: "the same, at the other end. The gate refuses a pair that is not \
              ordered, so these two are decided together",
        blocks: false,
        asked: true,
        insert: false,
    },
    Field {
        field: "reason_lower",
        table: "output",
        key: "reason_lower",
        shape: Shape::Prose,
        group: "the answer",
        ask: "why is the lower bound there",
        why: "a guard whose reason is not written gets deleted by the next person",
        blocks: true,
        asked: true,
        insert: false,
    },
    Field {
        field: "reason_upper",
        table: "output",
        key: "reason_upper",
        shape: Shape::Prose,
        group: "the answer",
        ask: "why is the upper bound there",
        why: "the same, at the other end",
        blocks: true,
        asked: true,
        insert: false,
    },
    Field {
        field: "sense",
        table: "",
        key: "sense",
        shape: Shape::Choice(SENSES),
        group: "the contract",
        ask: "which way does this requirement bind",
        why: "'<=' means the achieved value must stay under the bound, '>=' that \
              it must reach it. Read the wrong way the closure still computes and \
              still reports a comfortable margin, for a spacecraft that is about \
              to be destroyed",
        blocks: false,
        asked: true,
        insert: false,
    },
    Field {
        field: "declared_value",
        table: "value",
        key: "number",
        shape: Shape::Number,
        group: "the contract",
        ask: "what is the declared number",
        why: "a declared row computes nothing and publishes this, converted from \
              the unit above. It is the whole of the row's content",
        blocks: false,
        asked: true,
        insert: false,
    },
    Field {
        field: "theory_why",
        table: "theory",
        key: "why",
        shape: Shape::Prose,
        group: "the derivation",
        ask: "why is it this relation and not another",
        why: "the relation says what; this says why, and it is what a reviewer \
              reads before deciding whether to believe the number",
        blocks: false,
        asked: true,
        insert: true,
    },
    Field {
        field: "theory_reading",
        table: "theory",
        key: "reading",
        shape: Shape::Prose,
        group: "the derivation",
        ask: "how should the answer be read",
        why: "a number with no reading gets quoted out of context. This is where \
              a row says what its answer does NOT mean",
        blocks: false,
        asked: true,
        insert: true,
    },
    Field {
        field: "explain_breaks",
        table: "explain",
        key: "breaks",
        shape: Shape::Prose,
        group: "where it breaks",
        ask: "where does the simple version stop being true",
        why: "every simplification is wrong somewhere, and saying where is what makes it \
              safe to use. Unwritten, the plain words get quoted as the physics (E4)",
        blocks: false,
        asked: true,
        insert: true,
    },
    Field {
        field: "explain_wrong",
        table: "explain",
        key: "wrong",
        shape: Shape::Prose,
        group: "where it breaks",
        ask: "what do people commonly get wrong about this, and what is true instead",
        why: "the gap most readers share. Named and corrected on the page, it is not \
              rediscovered by each of them (E5). Leave it blank if there is none",
        blocks: false,
        asked: true,
        insert: true,
    },
    Field {
        field: "method_text",
        table: "method",
        key: "text",
        shape: Shape::Code,
        group: "the method",
        ask: "the method: your relation as a few lines in the method language \
              (docs/PSEUDOCODE.md). It reads the inputs by their names, says every number's \
              unit, and ends every path with return or refuse",
        why: "the code the tool ships is generated from this, and it is run on your test \
              cases below, so a mistake in it, in your code or in the generated code shows \
              up as a case that disagrees — before the change reaches anyone",
        blocks: false,
        asked: true,
        insert: true,
    },
    // Not asked: intake stamps the filler's name whenever the method changes,
    // as it does the relation.
    Field {
        field: "method_by",
        table: "method",
        key: "by",
        shape: Shape::Line,
        group: "the method",
        ask: "who wrote the method",
        why: "a method is mathematics, and the page names the person who owns it",
        blocks: false,
        asked: false,
        insert: true,
    },
    Field {
        field: "author_name",
        table: "author",
        key: "name",
        shape: Shape::Line,
        group: "your code",
        ask: "who wrote the code below",
        why: "the cases are only as good as the code that produced them, and a reviewer \
              needs to know whose it is",
        blocks: false,
        asked: true,
        insert: true,
    },
    Field {
        field: "author_language",
        table: "author",
        key: "language",
        shape: Shape::Choice(LANGUAGES),
        group: "your code",
        ask: "what language is it in",
        why: "what will run it again: Python and Octave can be rerun by the pipeline, the \
              rest are kept and read",
        blocks: false,
        asked: true,
        insert: true,
    },
    Field {
        field: "author_entry",
        table: "author",
        key: "entry",
        shape: Shape::Line,
        group: "your code",
        ask: "which function in it is this node",
        why: "a file often holds several functions; this names the one the cases call",
        blocks: false,
        asked: true,
        insert: true,
    },
    Field {
        field: "author_code",
        table: "author",
        key: "code",
        shape: Shape::Code,
        group: "your code",
        ask: "your code — the implementation you wrote and tested, pasted whole",
        why: "the first of the three statements of this node. Your cases come from it; the \
              method and the generated code are checked against them",
        blocks: false,
        asked: true,
        insert: true,
    },
    Field {
        field: "author_test_code",
        table: "author",
        key: "test_code",
        shape: Shape::Code,
        group: "your code",
        ask: "your test code — the script that ran your code on each test case and printed \
              the results",
        why: "so the cases can be produced again, by you or by the pipeline, rather than \
              taken on trust",
        blocks: false,
        asked: true,
        insert: true,
    },
    Field {
        field: "author_how_run",
        table: "author",
        key: "how_run",
        shape: Shape::Prose,
        group: "your code",
        ask: "where and how you ran it: the tool and its version, the machine, the date",
        why: "a number with no record of how it was made cannot be made again",
        blocks: false,
        asked: true,
        insert: true,
    },
    // Not asked: intake writes the filler's name here whenever a form changes
    // the plain words, so the page can say whose words they are.
    Field {
        field: "explain_by",
        table: "explain",
        key: "by",
        shape: Shape::Line,
        group: "said simply",
        ask: "who wrote the plain words",
        why: "the page names who said it, so a draft is never read as the owner's word",
        blocks: false,
        asked: false,
        insert: true,
    },
];

/// One field of the form, by name.
pub fn field(name: &str) -> Option<&'static Field> {
    FIELDS.iter().find(|f| f.field == name)
}

/// The form field that lives at one table and key, if any. The reverse of
/// `place`, for reading a pasted sheet.
pub fn field_at(table: &str, key: &str) -> Option<&'static str> {
    FIELDS
        .iter()
        .find(|f| f.table == table && f.key == key)
        .map(|f| f.field)
}

/// The tables this form writes into. A paste's `[maths]` is a container, not a
/// key, and reporting it as unwritable told a reader their whole relation had
/// been dropped.
pub fn tables() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for f in FIELDS {
        if !f.table.is_empty() && !out.contains(&f.table) {
            out.push(f.table);
        }
    }
    out
}

/// The groups, in the order the form asks them.
pub fn groups() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for f in FIELDS {
        if f.asked && !out.contains(&f.group) {
            out.push(f.group);
        }
    }
    out
}

/// Which of the nine required fields are still blank.
///
/// `docs` refuses to emit a scaffold while any of these is open: the signature
/// needs the type and the unit, and a bound with no reason is a guard the next
/// person deletes. Derived from `FIELDS`, so the form and the generator cannot
/// disagree about what a finished sheet is — they read one list.
pub fn unfilled(sh: &Sheet) -> Vec<&'static str> {
    FIELDS
        .iter()
        .filter(|f| f.blocks && value(sh, f.field).trim().is_empty())
        .map(|f| f.field)
        .collect()
}

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
            jq(&e)
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

/// A hash of the sheet's bytes, for detecting a concurrent edit.
///
/// NOT `sheet_hash`, and the difference matters. `sheet_hash` covers what a
/// reader would call the node's meaning and deliberately leaves out formatting,
/// comments and notes, so tidying a sentence does not invalidate every artefact
/// downstream. That makes it exactly wrong for this job: of the nine fields this
/// form writes, `label`, `symbol`, `reason_lower` and `reason_upper` are all
/// outside it, so two people editing a bound's reason would both see the same
/// `sheet_hash` and the second would overwrite the first without either being
/// told. This moves whenever the file moves.
pub fn file_hash(text: &str) -> String {
    crate::short_hex(crate::fnv1a(text))
}

/// Where each form field lives in the file: its table, and its key.
///
/// An empty table means a top-level key. Read from `FIELDS`, which is the one
/// place a field is described.
pub fn place(field: &str) -> Option<(&'static str, &'static str)> {
    self::field(field).map(|f| (f.table, f.key))
}

/// Every field `structural` refuses, in the order a reader meets them.
///
/// The list the form shows as locked and the manual lists as locked. It sits
/// beside `structural` so the two are one edit apart, and a test holds them to
/// each other: every name here must be refused, with a reason.
pub const LOCKED: &[&str] = &[
    "id",
    "folder",
    "parent",
    "order",
    "layer",
    "kind",
    "subsystem",
    "owner",
    "tier",
    "state",
];

/// The fields a face may never write, and why.
///
/// Not a blocklist of names but the reason each one is refused, because a
/// reader who is told "no" and not "why" goes looking for a way round.
pub fn structural(field: &str) -> Option<&'static str> {
    Some(match field {
        "id" | "folder" => {
            "the identifier is the folder and the variable name; renaming it is a move"
        }
        "parent" => "the parent is the tree's shape — moving a row moves everyone who reads it",
        "order" => {
            "order decides position among siblings, and the block may be packed solid; \
                    inserting renumbers its neighbours"
        }
        "layer" => "the layer decides which contract the row sits under",
        "kind" => {
            "kind decides whether the row declares a value or computes one, which changes \
                   what is generated for it"
        }
        "subsystem" | "owner" => {
            "ownership is generated into CODEOWNERS and decides who reviews it"
        }
        "tier" => "it changes what the gate demands of the row",
        "state" => {
            "a row's state decides whether anything is generated from it at all. It is \
             moved by publishing the row, which checks the whole sheet first, not by \
             typing into a box"
        }
        _ => return None,
    })
}
