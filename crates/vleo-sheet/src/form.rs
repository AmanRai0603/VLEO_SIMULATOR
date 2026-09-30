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
    fn bare(&self) -> bool {
        matches!(self, Shape::Number | Shape::Count | Shape::Inputs)
    }
    /// Whether an existing multi-line block is something this shape can hold.
    fn may_be_prose(&self) -> bool {
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

/// The key a line assigns, and what it assigns, or nothing.
///
/// A key is a bare name. A line whose left side has a space, a tab, a hash or a
/// bracket in it is a comment, a table header or prose — never an assignment —
/// which is why `# unit = "Foot"` needs no special case anywhere: the name it
/// yields is `# unit`, and that is not a key.
fn key_and_rhs(line: &str) -> Option<(&str, &str)> {
    let (lhs, rhs) = line.trim_start().split_once('=')?;
    let name = lhs.trim();
    if name.is_empty() || name.contains([' ', '\t', '#', '[']) {
        return None;
    }
    Some((name, rhs.trim()))
}

/// Whether this right-hand side opens a multi-line string that later lines
/// continue. `"""x"""` on one line does not.
fn opens_prose(rhs: &str) -> bool {
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
fn window(text: &str, table: &str) -> Option<(usize, usize)> {
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
fn assignments(text: &str, table: &str, key: &str) -> Vec<(usize, usize, String)> {
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
fn trailing_comment(rhs: &str) -> String {
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
fn has_key(text: &str, table: &str, key: &str) -> Option<(usize, usize)> {
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
fn ensure_table(text: &str, table: &str) -> Result<String, String> {
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
fn written(shape: &Shape, value: &str) -> String {
    if shape.bare() {
        value.to_string()
    } else {
        toml_quote(value)
    }
}

/// A TOML string literal. Prefers a basic string, falls back to a literal one
/// where the value has backslashes worth keeping as typed.
fn toml_quote(v: &str) -> String {
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

/// Every name that is an assistant, not a person, lowercased.
///
/// A relation is confirmed by a person who has read it against its source — the
/// developer who applies a node form, or who writes the relation in. An
/// assistant may implement a relation a person supplied; it may never be the one
/// who supplied it. These are the names an assistant arrives under.
pub fn agent_identities(_root: &std::path::Path) -> Vec<String> {
    [
        "claude",
        "agent",
        "assistant",
        "copilot",
        "chatgpt",
        "gpt",
        "gemini",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Who this checkout says it is, from `git config user.name`.
///
/// NOT typed into the form. An attribution a person types is a name they chose
/// for that box; this is the name their commits already carry, so the sheet and
/// the history agree about who did it and nobody can put a colleague's name on
/// their own work by typing it.
///
/// It is not authentication and this does not pretend otherwise: anyone who can
/// edit a checkout can edit its git config. What it removes is the casual case —
/// typing somebody else's name into a text box — and it makes the sheet's
/// attribution and the commit's author the same claim rather than two.
pub fn git_identity(root: &std::path::Path) -> Result<String, String> {
    let out = std::process::Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(["config", "user.name"])
        .output()
        .map_err(|e| format!("git could not be run: {e}"))?;
    let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if name.is_empty() {
        return Err(
            "this checkout has no `git config user.name`, so there is no name to put against \
             the relation. Set it — `git config user.name \"Your Name\"` — and the sheet will \
             carry the same name your commits do. Nothing was written."
                .into(),
        );
    }
    Ok(name)
}

/// Whether this attribution is an assistant's, and so must never be written.
///
/// An assistant may never supply mathematics. Stated as a sentence that is a hope;
/// here it is a fact about what can reach the file — and it has to hold at every
/// face, or the browser becomes the way round a rule the terminal enforces.
pub fn refuse_agent_attribution(root: &std::path::Path, who: &str) -> Result<(), String> {
    let lower = who.trim().to_lowercase();
    if lower.is_empty() {
        return Err(
            "an attribution cannot be blank — it takes the name of a person who has \
                    read the relation against its source and is prepared to own it"
                .into(),
        );
    }
    for bad in agent_identities(root) {
        if lower == bad
            || lower.starts_with(&format!("{bad} "))
            || lower.contains(&format!("{bad}/"))
        {
            return Err(format!(
                "refused: '{who}' is an assistant's name. An assistant may never supply mathematics, and this \
                 field is the only thing that can tell whether one did. It takes the name of a \
                 person who has read the relation against its source and is prepared to own it. \
                 Nothing was written."
            ));
        }
    }
    Ok(())
}

/// Today, as the sheets write it (vleo_data::clock).
fn today() -> String {
    vleo_data::clock::today()
}

/// Put a name against the relation, replacing whatever was there.
///
/// WHEN THE RELATION CHANGES THE ATTRIBUTION MUST MOVE WITH IT. The old name
/// was against the old mathematics; leaving it on the new attributes work to
/// somebody who never saw it, which is worse than either having no name or
/// having the editor's.
pub(crate) fn stamp_relation(text: &str, who: &str) -> Result<String, String> {
    set(text, "confirmed_by", &format!("{who} / {}", today()))
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
fn code_text(value: &str) -> String {
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
fn inputs_text(value: &str) -> Result<String, String> {
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

/// What a save did, or why it did nothing.
pub enum Saved {
    /// Written, regenerated and gated. Carries the row's new sheet hash.
    Ok {
        /// What the next save must send back.
        file_hash: String,
        /// Whether the row's MEANING moved — a caption or a bound's reason can
        /// change without this changing, and that is deliberate.
        sheet_hash: String,
        regenerated: usize,
    },
    /// The editor started from a version that is no longer current. Carries the
    /// hash it should have started from, so the face can show what changed
    /// rather than overwrite it.
    Stale { current: String },
    /// Refused, and nothing was written.
    Refused(String),
}

/// Change one field of one sheet, and leave the tree consistent or untouched.
///
/// The whole transaction, so it can be tested without an HTTP server:
///
///   1  the field is one the form writes, and is not structural
///   2  `base` is the hash the editor started from — a stale one is refused
///      rather than overwritten, which is what makes two editors safe
///   3  an edit to the relation carries an attribution, and that attribution is
///      not an assistant's
///   4  the sheet is written atomically: a temporary file, then a rename, so a
///      reader never sees half a sheet
///   5  the row's artefacts are regenerated and the gate is run on it
///   6  ANY failure after the write restores the previous sheet. A tree left
///      half-edited by a browser is the thing this must never do
///
/// `rustfmt` must be on the path, because the generated Rust is formatted before
/// it is compared and a fallback to unformatted text would leave the tree
/// failing its own regeneration diff. Refused up front rather than discovered
/// afterwards.
pub fn save(root: &std::path::Path, id: &str, field: &str, value: &str, base: &str) -> Saved {
    if let Some(why) = structural(field) {
        return Saved::Refused(format!("'{field}' is not editable here: {why}"));
    }
    if place(field).is_none() {
        return Saved::Refused(format!("'{field}' is not a field this form writes"));
    }
    if let Err(e) = value_allowed(field, value) {
        return Saved::Refused(e);
    }
    if let Some(why) = rustfmt_refusal() {
        return Saved::Refused(why);
    }

    let tree = match crate::load::load_all(root) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(format!("the tree does not load: {e}")),
    };
    let Some(sh) = tree.sheets.get(id) else {
        return Saved::Refused(format!("no node '{id}'"));
    };
    let path = sh.dir.join("node.toml");
    let before = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(format!("{}: {e}", path.display())),
    };
    let current = file_hash(&before);
    if base != current {
        return Saved::Stale { current };
    }
    // An assistant may never supply mathematics, at any face. The name is the
    // checkout's own — see `git_identity` — so it is the same one the commit
    // will carry rather than whatever was typed into a box.
    if field == "expression" || field == "confirmed_by" {
        let who = match git_identity(root) {
            Ok(w) => w,
            Err(e) => return Saved::Refused(e),
        };
        if let Err(e) = refuse_agent_attribution(root, &who) {
            return Saved::Refused(e);
        }
    }

    let after = match set(&before, field, value) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(e),
    };
    // AND THE NAME IS WRITTEN, NOT ONLY CHECKED. The identity was verified
    // above and then went nowhere, so a relation saved through the face came
    // out with `confirmed_by` still blank — a relation with nobody against it,
    // indistinguishable from one an assistant wrote, which is the exact thing that
    // field exists to tell apart.
    let after = if field == "expression" {
        let who = match git_identity(root) {
            Ok(w) => w,
            Err(e) => return Saved::Refused(e),
        };
        match stamp_relation(&after, &who) {
            Ok(t) => t,
            Err(e) => return Saved::Refused(e),
        }
    } else {
        after
    };
    commit_edit(root, id, &path, &before, after, false)
}

/// Why an edit cannot be written here, if `rustfmt` is not on the path.
///
/// The generated Rust is formatted before it is compared, and a fallback to
/// unformatted text would leave the tree failing its own regeneration diff. So
/// every edit refuses up front rather than discovering it afterwards.
pub(crate) fn rustfmt_refusal() -> Option<String> {
    let missing = std::process::Command::new("rustfmt")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| !s.success())
        .unwrap_or(true);
    missing.then(|| {
        "rustfmt is not on the path. The generated Rust is formatted before it is compared, \
         so saving without it would leave the tree failing its own regeneration check. \
         Nothing was written."
            .to_string()
    })
}

/// Write the edit, regenerate the row, gate it — or put everything back.
///
/// The tail of every edit, whether it moved one field or a whole repeated
/// block. Shared because the failure path is the part that matters and a second
/// copy of it would be a second chance to get the restore wrong.
///
/// `whole_tree` runs the assembly validations as well. They are not free — they
/// walk 1396 rows — so they run for the one edit that can break a row which is
/// not this one: an input is an edge, and whether an edge closes a loop in the
/// derivation graph is a question only the whole graph can answer.
///
/// ANY FAILURE AFTER THE WRITE RESTORES THE PREVIOUS SHEET. A tree left
/// half-edited by a browser is the thing this must never do.
pub(crate) fn commit_edit(
    root: &std::path::Path,
    id: &str,
    path: &std::path::Path,
    before: &str,
    after: String,
    whole_tree: bool,
) -> Saved {
    // Which generated files the row had BEFORE the edit. Taken before anything
    // is written, because it is the answer to "what may a restore delete".
    let dir = path.parent().unwrap_or(std::path::Path::new("."));
    let existed: Vec<std::path::PathBuf> = GENERATED
        .iter()
        .map(|n| dir.join(n))
        .filter(|p| p.exists())
        .collect();

    if let Err(e) = write_atomic(path, &after) {
        return Saved::Refused(e);
    }

    // Restoring the sheet is not enough on its own: once the artefacts have
    // been regenerated from the rejected edit, putting only node.toml back
    // leaves the tree failing its own regeneration check — the exact state this
    // whole path exists to avoid. So the artefacts are regenerated from the
    // restored sheet too.
    //
    // AND WHAT THE EDIT GENERATED THAT WAS NOT THERE BEFORE IS REMOVED. A
    // refused publish showed why: publishing a seeded row makes the generator
    // write its model, contract, module and evidence for the first time, and
    // regenerating the restored — still seeded — sheet writes only its page. The
    // four new files stayed behind under a message saying nothing had changed,
    // and the next "put these edits on a branch" would have committed them.
    // Only a generated file the row did not have before is removed; one that
    // existed is regenerated, which keeps whatever Rust its holes hold.
    let restore = |e: String| -> Saved {
        let _ = write_atomic(path, before);
        for n in GENERATED {
            let p = dir.join(n);
            if p.exists() && !existed.contains(&p) {
                let _ = std::fs::remove_file(&p);
            }
        }
        let put_back = crate::load::load_all(root)
            .ok()
            .and_then(|t| t.sheets.get(id).map(|s| regenerate(s, &t)));
        let lost = match put_back {
            Some(Err(w)) => format!(" — AND THE ARTEFACTS COULD NOT BE PUT BACK: {w}"),
            None => " — AND THE TREE WOULD NOT RELOAD TO PUT THE ARTEFACTS BACK".into(),
            Some(Ok(_)) => String::new(),
        };
        Saved::Refused(format!(
            "{e} — the sheet was restored, nothing changed{lost}"
        ))
    };
    let tree = match crate::load::load_all(root) {
        Ok(t) => t,
        Err(e) => return restore(format!("the edit does not parse: {e}")),
    };
    let Some(sh) = tree.sheets.get(id) else {
        return restore("the row vanished from the tree after the edit".into());
    };
    // REGENERATE BEFORE GATING, not after. One of the gate's own checks is that
    // every artefact matches what the sheet generates, so gating a freshly
    // written sheet whose artefacts are still the old ones fails every time —
    // and fails for a reason that has nothing to do with the edit.
    let n = match regenerate(sh, &tree) {
        Ok(n) => n,
        Err(e) => return restore(e),
    };
    let mut failed: Vec<String> = crate::gate::gate_node(sh, &tree)
        .iter()
        .filter(|c| c.failed())
        .map(|c| match &c.verdict {
            crate::gate::Verdict::Fail(w) => format!("{}: {w}", c.name),
            _ => c.name.to_string(),
        })
        .collect();
    if whole_tree && failed.is_empty() {
        failed = crate::gate::validate_tree(&tree)
            .iter()
            .filter(|c| c.failed())
            .map(|c| match &c.verdict {
                crate::gate::Verdict::Fail(w) => format!("{}: {w}", c.name),
                _ => c.name.to_string(),
            })
            .collect();
    }
    if !failed.is_empty() {
        return restore(format!("the gate refuses it — {}", failed.join("; ")));
    }
    Saved::Ok {
        file_hash: std::fs::read_to_string(path)
            .map(|t| file_hash(&t))
            .unwrap_or_default(),
        sheet_hash: crate::short_hex(sh.sheet_hash),
        regenerated: n,
    }
}

/// Every file the per-row generators write. `regenerate` writes the first four
/// only for a published row; the last two for every row.
const GENERATED: &[&str] = &[
    "model.rs",
    "contract.rs",
    "mod.rs",
    "evidence.rs",
    "page.html",
    "meta.json",
];

/// A temporary file then a rename, so a reader never sees half a sheet.
fn write_atomic(path: &std::path::Path, text: &str) -> Result<(), String> {
    let tmp = path.with_extension("toml.writing");
    std::fs::write(&tmp, text).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
}

/// `regenerate`, for a test that has to put a row back after editing it.
pub fn regenerate_for_test(
    sh: &crate::model::Sheet,
    tree: &crate::load::Tree,
) -> Result<usize, String> {
    regenerate(sh, tree)
}

/// The six per-node generators, for one row. The same set `xtask docs` writes.
fn regenerate(sh: &crate::model::Sheet, tree: &crate::load::Tree) -> Result<usize, String> {
    let holes = crate::load::read_holes(&sh.dir);
    let gaps = crate::emit::gap_pass(sh, &holes);
    let artefacts: Vec<(&str, String)> = if sh.is_seeded() {
        vec![
            ("page.html", crate::page::fragment(sh, &holes, tree)),
            ("meta.json", crate::emit::meta_json(sh, &gaps)),
        ]
    } else {
        vec![
            ("model.rs", crate::emit::model_rs(sh, &holes)),
            ("contract.rs", crate::emit::contract_rs(sh)),
            ("mod.rs", crate::emit::mod_rs(sh)),
            ("evidence.rs", crate::emit::evidence_rs(sh)),
            ("page.html", crate::page::fragment(sh, &holes, tree)),
            ("meta.json", crate::emit::meta_json(sh, &gaps)),
        ]
    };
    let mut n = 0;
    for (name, text) in artefacts {
        let text = if name.ends_with(".rs") {
            crate::gate::formatted(&text)
        } else {
            text
        };
        let p = sh.dir.join(name);
        let same = std::fs::read_to_string(&p)
            .map(|o| o == text)
            .unwrap_or(false);
        if !same {
            std::fs::write(&p, &text).map_err(|e| format!("{}: {e}", p.display()))?;
            n += 1;
        }
    }
    // The node's method lives in the kernel, not beside the sheet, and the
    // model this just wrote calls it — so it is written in the same step.
    n += crate::emit::sync_methods(tree)?;
    Ok(n)
}

// ─── the repeated blocks ─────────────────────────────────────────────────────
//
// A sheet's inputs, algorithm steps, theory steps and assumptions are
// `[[table]]` arrays, and they are not fields. A field is replaced; a block is
// ADDED and REMOVED as well, and adding or removing one can break something a
// scalar edit never could — a numbered hole in generated Rust, a hand-written
// body that binds a name, an edge that closes a loop in the derivation graph.
//
// So they have their own table, their own operations and their own refusals.

/// One key inside a repeated block.
pub struct Column {
    pub key: &'static str,
    pub shape: Shape,
    pub ask: &'static str,
    /// A block with this blank says nothing.
    pub required: bool,
    /// Written by this module and never typed. A step's `number` is the
    /// identity of a numbered hole in `model.rs`; a person retyping it would
    /// reattach somebody's Rust to a different step.
    pub managed: bool,
}

/// Where a block may be added and removed.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Blocks {
    /// Added, edited and removed, subject to the checks in `save_block`.
    Free,
    /// Added at the end and removed from the end only.
    ///
    /// The blocks are numbered and the number is a hole identifier in generated
    /// Rust. Appending leaves every existing number where it is; inserting in
    /// the middle renumbers the ones after it, which moves each hand-written
    /// body onto a different step without changing a line of it. That is not an
    /// edit anybody would notice in review.
    EndOnly,
}

/// One repeated block of a sheet.
pub struct Array {
    pub name: &'static str,
    /// Its TOML path, which is what `[[…]]` carries.
    pub path: &'static str,
    pub label: &'static str,
    pub why: &'static str,
    pub columns: &'static [Column],
    pub blocks: Blocks,
    /// The table a first block goes after, when the sheet has none yet.
    pub after: &'static str,
    /// The group whose rows alone carry this block, or empty for every row.
    /// A risk is registered on a risk-register row and nowhere else.
    pub only_under: &'static str,
}

impl Array {
    /// Whether this block belongs on this row's form.
    pub fn applies(&self, sh: &Sheet) -> bool {
        self.only_under.is_empty() || sh.parent == self.only_under
    }
}

/// EVERY REPEATED BLOCK THIS FORM WRITES.
pub const ARRAYS: &[Array] = &[
    Array {
        name: "input",
        path: "input",
        label: "what this row reads",
        why: "the consumer declares what it expects, and assembly refuses a mismatch against \
              what the producer publishes. It is also the derivation graph: an input is an \
              edge, and the edges are what decide what runs before what",
        columns: &[
            Column {
                key: "binding",
                shape: Shape::Line,
                ask: "the name it goes by inside this row",
                required: true,
                managed: false,
            },
            Column {
                key: "var",
                shape: Shape::RowId,
                ask: "which row answers it",
                required: true,
                managed: false,
            },
            Column {
                key: "type",
                shape: Shape::Quantity,
                ask: "the quantity this row expects it to be",
                required: true,
                managed: false,
            },
        ],
        blocks: Blocks::Free,
        after: "output",
        only_under: "",
    },
    Array {
        name: "algorithm",
        path: "algorithm.step",
        label: "the steps",
        why: "each step becomes one numbered hole in the generated model, and the hole is \
              where a person writes the few typed lines that compose relations from the \
              kernel. The step's text is the comment above it",
        columns: &[
            Column {
                key: "number",
                shape: Shape::Count,
                ask: "",
                required: true,
                managed: true,
            },
            Column {
                key: "text",
                shape: Shape::Prose,
                ask: "what this step does",
                required: true,
                managed: false,
            },
            Column {
                key: "binds",
                shape: Shape::Line,
                ask: "the name its result is bound to",
                required: true,
                managed: false,
            },
            Column {
                key: "type",
                shape: Shape::Quantity,
                ask: "and its quantity",
                required: true,
                managed: false,
            },
        ],
        blocks: Blocks::EndOnly,
        after: "output",
        only_under: "",
    },
    Array {
        name: "theory",
        path: "theory.step",
        label: "how the relation was arrived at",
        why: "the expression says what the relation is; these say how it was got to. A \
              relation an assistant invented carries a citation just as convincingly, and the \
              derivation is what a reviewer reads instead of taking the citation's word",
        columns: &[
            Column {
                key: "text",
                shape: Shape::Prose,
                ask: "the step of the argument",
                required: true,
                managed: false,
            },
            Column {
                key: "math",
                shape: Shape::Line,
                ask: "and the line of maths, if there is one",
                required: false,
                managed: false,
            },
        ],
        blocks: Blocks::Free,
        after: "theory",
        only_under: "",
    },
    Array {
        name: "assumption",
        path: "assumption",
        label: "what has to be true for this to hold",
        why: "an assumption nobody wrote down is one nobody can check. Each says what it \
              assumes AND the case it fails in, because an assumption with no failure case \
              is a sentence rather than a warning",
        columns: &[
            Column {
                key: "text",
                shape: Shape::Prose,
                ask: "what is assumed",
                required: true,
                managed: false,
            },
            Column {
                key: "fails_when",
                shape: Shape::Prose,
                ask: "and when that stops being true",
                required: true,
                managed: false,
            },
        ],
        blocks: Blocks::Free,
        // NOT `theory`. 1322 of 1396 rows have no `[theory]` at all, so a first
        // assumption on one of them had nothing to go after and was refused —
        // for a table that is optional. `[maths]` is on every row, and
        // `insert_point` still prefers the derivation when there is one, so the
        // reading order of an authored sheet is kept where it exists.
        after: "maths",
        only_under: "",
    },
    Array {
        name: "risk",
        path: "risk",
        label: "the risks this row registers",
        why: "a risk is registered once, here, and moved only by node versions — a risk is \
              reduced because something was tested, and the version that tested it is the \
              record of how. Which risk-register row holds a risk says what kind it is",
        columns: &[
            Column {
                key: "id",
                shape: Shape::Line,
                ask: "its id: R- and a number, never reused",
                required: true,
                managed: false,
            },
            Column {
                key: "title",
                shape: Shape::Line,
                ask: "what could go wrong, in a line",
                required: true,
                managed: false,
            },
            Column {
                key: "level",
                shape: Shape::Choice(crate::derisk::LEVELS),
                ask: "how serious it is now, L1 (least) to L5",
                required: true,
                managed: false,
            },
            Column {
                key: "owner",
                shape: Shape::Line,
                ask: "who owns it",
                required: true,
                managed: false,
            },
            Column {
                key: "since",
                shape: Shape::Line,
                ask: "registered on (YYYY-MM-DD)",
                required: true,
                managed: false,
            },
            Column {
                key: "why",
                shape: Shape::Prose,
                ask: "what it would cost if it happened",
                required: true,
                managed: false,
            },
        ],
        blocks: Blocks::Free,
        after: "maths",
        only_under: crate::derisk::REGISTER,
    },
    Array {
        name: "case",
        path: "case",
        label: "your test cases",
        why: "inputs and the answer YOUR code gave, or that your code refuses them. They \
              decide: the method is run on each one before you send the form, and the code \
              the tool generates must reproduce every one before the node is connected to \
              anything. At least three answers and one refusal",
        columns: &[
            Column {
                key: "label",
                shape: Shape::Line,
                ask: "what this case is, in a few words — 'lowest altitude', 'a storm'",
                required: true,
                managed: false,
            },
            Column {
                key: "refuse",
                shape: Shape::Choice(YES_NO),
                ask: "must the node refuse this case",
                required: true,
                managed: false,
            },
            Column {
                key: "expect",
                shape: Shape::Number,
                ask: "the answer your code gave, in SI",
                required: false,
                managed: false,
            },
            Column {
                key: "tolerance",
                shape: Shape::Number,
                ask: "how close is close enough, as a fraction (1e-6 is one part in a million)",
                required: false,
                managed: false,
            },
            Column {
                key: "inputs",
                shape: Shape::Inputs,
                ask: "each input's value, in SI",
                required: true,
                managed: false,
            },
        ],
        blocks: Blocks::Free,
        after: "maths",
        only_under: "",
    },
    Array {
        name: "flight",
        path: "flight",
        label: "flight software kept with this node",
        why: "on-board code that implements this node, stored with its test beside the \
              relation it implements, so the two are reviewed and versioned together",
        columns: &[
            Column {
                key: "name",
                shape: Shape::Line,
                ask: "its file name, as in the flight tree",
                required: true,
                managed: false,
            },
            Column {
                key: "language",
                shape: Shape::Line,
                ask: "its language",
                required: true,
                managed: false,
            },
            Column {
                key: "purpose",
                shape: Shape::Prose,
                ask: "what it does on board, and which part of this node it implements",
                required: true,
                managed: false,
            },
            Column {
                key: "code",
                shape: Shape::Code,
                ask: "the code",
                required: true,
                managed: false,
            },
            Column {
                key: "test_code",
                shape: Shape::Code,
                ask: "its test",
                required: false,
                managed: false,
            },
            Column {
                key: "test_result",
                shape: Shape::Prose,
                ask: "what the test gave when it was last run, and where",
                required: false,
                managed: false,
            },
        ],
        blocks: Blocks::Free,
        after: "maths",
        only_under: "",
    },
];

/// One repeated block, by name.
pub fn array(name: &str) -> Option<&'static Array> {
    ARRAYS.iter().find(|a| a.name == name)
}

/// One line of a sheet, and whether it is inside a multi-line string.
///
/// EVERYTHING TEXTUAL HERE NEEDS THIS. A line beginning `[` inside somebody's
/// paragraph is not a table header and a line reading `x = 1` inside one is not
/// an assignment; treating either as what it looks like edits a reason and
/// leaves the field alone.
struct Row {
    start: usize,
    end: usize,
    prose: bool,
}

fn scan(text: &str) -> Vec<Row> {
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

/// The byte range of each `[[path]]` block, in the order they appear.
///
/// A block runs from its own header line to the next table header of any kind,
/// so whatever trails it — a blank line, a comment about the next one — travels
/// with it. That is what makes removing one leave a file somebody would have
/// written.
fn array_blocks(text: &str, path: &str) -> Vec<(usize, usize)> {
    let header = format!("[[{path}]]");
    let mut out: Vec<(usize, usize)> = Vec::new();
    let mut open: Option<usize> = None;
    for r in scan(text) {
        if r.prose {
            continue;
        }
        let t = text[r.start..r.end].trim();
        if !t.starts_with('[') {
            continue;
        }
        if let Some(s) = open.take() {
            out.push((s, r.start));
        }
        if t == header {
            open = Some(r.start);
        }
    }
    if let Some(s) = open {
        out.push((s, text.len()));
    }
    out
}

/// Where a new block of this array goes.
///
/// After the last one of its kind, so a sheet reads in the order it was written.
/// When there are none, after the table the array belongs under — which for a
/// theory step means creating `[theory]` first, exactly as a scalar theory field
/// does.
fn insert_point(text: &str, a: &Array) -> Result<usize, String> {
    if let Some((_, end)) = array_blocks(text, a.path).last() {
        return Ok(*end);
    }
    // An assumption reads after the derivation it qualifies, when there is one.
    if a.path == "assumption" {
        if let Some((_, end)) = array_blocks(text, "theory.step").last() {
            return Ok(*end);
        }
        if let Some((_, end)) = window(text, "theory") {
            return Ok(end);
        }
    }
    // The author's cases read after their code, which produced them; flight
    // software after the cases.
    if a.path == "case" || a.path == "flight" {
        if a.path == "flight" {
            if let Some((_, end)) = array_blocks(text, "case").last() {
                return Ok(*end);
            }
        }
        for t in ["author", "method"] {
            if let Some((_, end)) = window(text, t) {
                return Ok(end);
            }
        }
        if let Some((_, end)) = array_blocks(text, "theory.step").last() {
            return Ok(*end);
        }
    }
    // An algorithm step reads after the inputs it consumes.
    if a.path == "algorithm.step" {
        if let Some((_, end)) = array_blocks(text, "input").last() {
            return Ok(*end);
        }
    }
    let (_, end) = window(text, a.after)
        .ok_or_else(|| format!("this sheet has no [{}] table to put it after", a.after))?;
    // `window` ends at the newline before the next header, so a block written at
    // that offset needs the newline back in front of it. `write_block` adds it.
    Ok(end)
}

/// One block's text, from its keys.
fn write_block(a: &Array, values: &[(&str, String)]) -> String {
    let mut o = format!("\n\n[[{}]]\n", a.path);
    for c in a.columns {
        if let Some((_, v)) = values.iter().find(|(k, _)| *k == c.key) {
            if v.trim().is_empty() && !c.required {
                continue;
            }
            o.push_str(&format!("{} = {}\n", c.key, written(&c.shape, v)));
        }
    }
    o
}

/// Change one repeated block, and leave the tree consistent or untouched.
///
/// `op` is `add`, `set` or `remove`. `values` carries the keys to write for an
/// add, or the one key and its value for a set.
///
/// This is the half that needs the tree: what the sheet already reads, and what
/// the hand-written hole bodies in `model.rs` depend on. The textual half is
/// `block_text`, which can be tested against a sheet in a string. The
/// transaction is `commit_edit`'s — the same stale-hash check, atomic write,
/// regenerate-then-gate and restore-on-any-failure that a scalar edit gets.
pub fn save_block(
    root: &std::path::Path,
    id: &str,
    name: &str,
    op: &str,
    index: usize,
    values: &[(&str, String)],
    base: &str,
) -> Saved {
    let Some(a) = array(name) else {
        return Saved::Refused(format!("'{name}' is not a repeated block this form writes"));
    };
    let tree = match crate::load::load_all(root) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(format!("the tree does not load: {e}")),
    };
    let Some(sh) = tree.sheets.get(id) else {
        return Saved::Refused(format!("no node '{id}'"));
    };
    let path = sh.dir.join("node.toml");
    let before = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(format!("{}: {e}", path.display())),
    };
    if base != file_hash(&before) {
        return Saved::Stale {
            current: file_hash(&before),
        };
    }

    // THE HOLES ARE THE THING A BLOCK EDIT CAN BREAK. A hole body is Rust
    // somebody wrote by hand, and the generated code around it is written from
    // these blocks: an input's `binding` is a parameter name, a step's `binds`
    // and `type` are the `let` the body has to satisfy. Change either under a
    // filled hole and the row stops compiling — which the gate cannot see,
    // because the gate never compiles anything.
    let holes = crate::load::read_holes(&sh.dir);
    let filled: Vec<(u32, &String)> = holes
        .iter()
        .filter(|(_, b)| !b.trim().is_empty())
        .map(|(n, b)| (*n, b))
        .collect();
    let mentions = |word: &str| -> Option<u32> {
        let word = word.trim();
        if word.is_empty() {
            return None;
        }
        filled
            .iter()
            .find(|(_, b)| {
                b.split(|c: char| !c.is_alphanumeric() && c != '_')
                    .any(|w| w == word)
            })
            .map(|(n, _)| *n)
    };
    let blocks = array_blocks(&before, a.path);
    let at = |i: usize| -> Option<(usize, usize)> { blocks.get(i).copied() };

    match op {
        "add" if a.path == "input" => {
            // V2 — each edge is declared once, by the end it changes. That is an
            // assembly validation, and this refuses it here so the reason names
            // the row rather than arriving as a rolled-back tree failure.
            if let Some((_, v)) = values.iter().find(|(k, _)| *k == "var") {
                if sh.inputs.iter().any(|i| &i.var == v) {
                    return Saved::Refused(format!(
                        "this row already reads '{v}'. Each edge is declared once, by the end it \
                         changes — a second declaration is two edges the graph cannot tell apart"
                    ));
                }
            }
        }
        "set" => {
            let Some((s0, s1)) = at(index) else {
                return Saved::Refused(no_such_block(a, blocks.len(), index));
            };
            // The keys a filled hole is written around.
            if let Some((key, _)) = values.first() {
                let guarded = (a.path == "input" && *key == "binding")
                    || (a.path == "algorithm.step" && matches!(*key, "binds" | "type"));
                if guarded {
                    let old = block_value(&before, s0, s1, key).unwrap_or_default();
                    if let Some(n) = mentions(&old) {
                        return Saved::Refused(format!(
                            "hole {n} is filled and its body uses `{}`. Changing it here would \
                             move the generated code around a body nobody has read since, and \
                             the gate never compiles anything, so nothing would catch it. Edit \
                             the hole and this together, in a checkout",
                            old.trim()
                        ));
                    }
                }
            }
        }
        "remove" => {
            let Some((s0, s1)) = at(index) else {
                return Saved::Refused(no_such_block(a, blocks.len(), index));
            };
            if a.path == "algorithm.step" {
                let n = (index + 1) as u32;
                if holes.get(&n).map(|b| !b.trim().is_empty()).unwrap_or(false) {
                    return Saved::Refused(format!(
                        "hole {n} has a body. Removing the step would delete the generated \
                         model's only home for it. Empty the hole first, in a checkout, so \
                         losing the Rust is its own reviewable change"
                    ));
                }
            }
            if a.path == "input" {
                let old = block_value(&before, s0, s1, "binding").unwrap_or_default();
                if let Some(n) = mentions(&old) {
                    return Saved::Refused(format!(
                        "hole {n} is filled and its body uses `{}`. Removing the input would \
                         take the parameter out of the generated signature and leave that body \
                         referring to nothing",
                        old.trim()
                    ));
                }
            }
        }
        _ => {}
    }

    let after = match block_text(&before, name, op, index, values) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(e),
    };
    // AN EDGE IS THE ONE EDIT THAT CAN BREAK A ROW THAT IS NOT THIS ONE. The
    // per-node gate asks whether each input resolves and agrees on type; whether
    // the edge closes a loop in the derivation graph is a question about the
    // whole graph, and only the assembly validations can ask it.
    commit_edit(root, id, &path, &before, after, a.path == "input")
}

fn no_such_block(a: &Array, have: usize, index: usize) -> String {
    format!(
        "this row has {have} {} block(s); there is no number {}",
        a.name,
        index + 1
    )
}

/// The textual half of a block edit: what the sheet becomes.
///
/// Separate from `save_block` so it can be tested against a sheet in a string.
/// What is here is everything decidable from the file itself — the shapes, the
/// required keys, the managed number, where a new block goes. What needs the
/// tree or the hole bodies stays in `save_block`.
pub fn block_text(
    text: &str,
    name: &str,
    op: &str,
    index: usize,
    values: &[(&str, String)],
) -> Result<String, String> {
    let a =
        array(name).ok_or_else(|| format!("'{name}' is not a repeated block this form writes"))?;
    let blocks = array_blocks(text, a.path);
    match op {
        // ADDED AT THE END, ALWAYS, whatever the array. Inserting in the middle
        // is not offered rather than refused: for a numbered array it would move
        // every hand-written hole body after the insertion onto a different
        // step, and for the others the order is a reading order somebody can
        // rearrange in a checkout where the diff is legible.
        "add" => {
            let mut vals: Vec<(&str, String)> = Vec::new();
            for c in a.columns {
                let v = if c.managed {
                    // The step number is this module's to assign: one past the
                    // last, which is the only value that leaves every existing
                    // hole where it is.
                    (blocks.len() + 1).to_string()
                } else {
                    values
                        .iter()
                        .find(|(k, _)| *k == c.key)
                        .map(|(_, v)| v.clone())
                        .unwrap_or_default()
                };
                if v.trim().is_empty() {
                    if c.required {
                        return Err(format!("a {} block needs `{}`: {}", a.name, c.key, c.ask));
                    }
                    continue;
                }
                vals.push((c.key, normalise_column(c, &v)?));
            }
            // A theory step needs the table its path sits under, exactly as a
            // scalar theory field does.
            let text = if a.path.starts_with("theory.") {
                ensure_table(text, "theory")?
            } else {
                text.to_string()
            };
            let at = insert_point(&text, a)?;
            let mut o = String::with_capacity(text.len() + 256);
            o.push_str(text[..at].trim_end_matches('\n'));
            o.push_str(&write_block(a, &vals));
            let tail = text[at..].trim_start_matches('\n');
            if !tail.is_empty() {
                o.push('\n');
                o.push_str(tail);
            }
            Ok(o)
        }
        "set" => {
            let &(s0, s1) = blocks
                .get(index)
                .ok_or_else(|| no_such_block(a, blocks.len(), index))?;
            let (key, value) = values.first().ok_or("nothing to set")?;
            let c = a
                .columns
                .iter()
                .find(|c| c.key == *key)
                .ok_or_else(|| format!("'{key}' is not a key of a {} block", a.name))?;
            if c.managed {
                return Err(format!(
                    "`{}` is written by the form and never typed: it is the identity of a \
                     numbered hole in the generated model, and retyping it would reattach \
                     somebody's Rust to a different step",
                    c.key
                ));
            }
            if c.required && value.trim().is_empty() {
                return Err(format!(
                    "a {} block needs `{}`: {}. Remove the whole block rather than emptying \
                     one of its keys",
                    a.name, c.key, c.ask
                ));
            }
            let v = normalise_column(c, value)?;
            set_in_block(text, s0, s1, c, &v)
        }
        "remove" => {
            let &(s0, s1) = blocks
                .get(index)
                .ok_or_else(|| no_such_block(a, blocks.len(), index))?;
            if a.blocks == Blocks::EndOnly && index + 1 != blocks.len() {
                return Err(format!(
                    "only the last {} block can be removed. Removing one from the middle \
                     renumbers every step after it, and each number is a hole holding \
                     somebody's Rust",
                    a.name
                ));
            }
            let mut o = String::with_capacity(text.len());
            o.push_str(text[..s0].trim_end_matches('\n'));
            o.push('\n');
            let tail = text[s1..].trim_start_matches('\n');
            if !tail.is_empty() {
                o.push('\n');
                o.push_str(tail);
            }
            Ok(o)
        }
        _ => Err(format!("'{op}' is not add, set or remove")),
    }
}

/// What one block currently says for a key.
///
/// Read through the TOML parser rather than by line, because this is a READ and
/// the parser is the authority on what a block says. It is writing that has to
/// be textual, to keep the comments a serialiser would drop.
fn block_value(text: &str, s0: usize, s1: usize, key: &str) -> Option<String> {
    let body = text[s0..s1].split_once('\n')?.1;
    let v: toml::Value = body.parse().ok()?;
    v.get(key).and_then(|x| x.as_str()).map(String::from)
}

/// Replace or add one key inside one block.
fn set_in_block(
    text: &str,
    s0: usize,
    s1: usize,
    c: &Column,
    value: &str,
) -> Result<String, String> {
    let body = &text[s0..s1];
    // The block's own lines, minus its header, with prose bodies skipped.
    let mut hit: Option<(usize, usize, String)> = None;
    let mut open: Option<(usize, bool)> = None;
    for r in scan(body) {
        let line = &body[r.start..r.end];
        match open {
            Some((start, wanted)) => {
                if let Some(i) = line.find("\"\"\"") {
                    if wanted {
                        hit = Some((start, r.end, trailing_comment(&line[i + 3..])));
                    }
                    open = None;
                }
            }
            None => {
                if let Some((name, rhs)) = key_and_rhs(line) {
                    if opens_prose(rhs) {
                        open = Some((r.start, name == c.key));
                    } else if name == c.key {
                        if hit.is_some() {
                            return Err(format!(
                                "`{}` is assigned twice in this block — refusing to guess which \
                                 one is meant",
                                c.key
                            ));
                        }
                        hit = Some((r.start, r.end, trailing_comment(rhs)));
                    }
                }
            }
        }
    }
    let written = format!("{} = {}", c.key, written(&c.shape, value));
    let mut o = String::with_capacity(text.len() + value.len());
    match hit {
        Some((a, b, comment)) => {
            o.push_str(&text[..s0 + a]);
            o.push_str(&written);
            o.push_str(&comment);
            o.push('\n');
            o.push_str(&text[s0 + b..]);
        }
        None => {
            // Absent and optional — `math` on a theory step is the case. Added
            // directly under the block's header, which is the one place in a
            // block that is unambiguous.
            let head = body
                .find('\n')
                .map(|i| s0 + i + 1)
                .ok_or_else(|| "this block has no body to write into".to_string())?;
            o.push_str(&text[..head]);
            o.push_str(&written);
            o.push('\n');
            o.push_str(&text[head..]);
        }
    }
    Ok(o)
}

/// A column's value as it must be written, or why it cannot be.
fn normalise_column(c: &Column, value: &str) -> Result<String, String> {
    let v = value.trim();
    match c.shape {
        Shape::Count => v
            .parse::<u32>()
            .map(|n| n.to_string())
            .map_err(|_| format!("`{}` is a whole number; '{v}' is not", c.key)),
        Shape::Quantity if !crate::is_quantity_name(v) => Err(format!(
            "'{v}' is not a quantity this system has. It is written straight into the \
             generated signature, so one that does not exist stops the tree compiling. One \
             of: {}",
            vleo_units::QUANTITIES.join(", ")
        )),
        Shape::RowId if v.is_empty() || v.contains(char::is_whitespace) => Err(format!(
            "'{v}' is not a row id — an id is one word, and whether it resolves is the gate's \
             question"
        )),
        Shape::Line if v.contains('\n') => {
            Err(format!("`{}` is one line; what was sent is not", c.key))
        }
        Shape::Prose => Ok(value.trim_end().trim_start_matches('\n').to_string()),
        Shape::Code => Ok(code_text(value)),
        Shape::Inputs => inputs_text(value),
        Shape::Number => {
            let n: f64 = v
                .parse()
                .map_err(|_| format!("`{}` is a number; '{v}' is not", c.key))?;
            if !n.is_finite() {
                return Err(format!("`{}` must be finite", c.key));
            }
            Ok(format!("{n:?}"))
        }
        Shape::Choice(options) if !options.contains(&v) => Err(format!(
            "`{}` is one of: {} — not '{v}'",
            c.key,
            options.join(", ")
        )),
        _ => Ok(v.to_string()),
    }
}

/// The `[view]` table, rewritten coherently.
///
/// NOT THREE FIELDS. A view's kind decides which other keys exist: a number has
/// none, a line has `over` and `points`, a bar has `y`. Offered as three
/// separate scalar fields the form would have to let somebody set `over` on a
/// row whose kind is `number`, where the key does not exist and means nothing,
/// and then set the kind afterwards — two saves, either order wrong. So the
/// whole table is written at once from the state the face sends.
///
/// `over` is the row the picture is drawn against, whatever the kind calls it:
/// the file says `over` for a line and `y` for a bar, and a face should not have
/// to know that.
///
/// It REFUSES A TABLE IT DOES NOT UNDERSTAND. A `[view]` carrying a comment, a
/// key outside the set, or a heatmap — which has two axes and a grid and is not
/// something this offers — is reported rather than flattened. This is the one
/// writer here that replaces a whole table instead of one value, so it is the
/// one place something can be lost without the loss appearing anywhere.
///
/// Whether the row being drawn against EXISTS is not asked here: that needs the
/// tree, and `save_view` asks it.
pub fn view_rewrite(text: &str, kind: &str, over: &str, points: &str) -> Result<String, String> {
    if !VIEW_KINDS.contains(&kind) {
        return Err(format!(
            "'{kind}' is not a way this tool draws an answer. One of: {}",
            VIEW_KINDS.join(", ")
        ));
    }
    let (from, to) = window(text, "view").ok_or(
        "this sheet has no [view] table, which every sheet should have. That is a malformed \
         sheet and not something a form should paper over — edit it directly",
    )?;
    for line in text[from..to].lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with('#') {
            return Err(format!(
                "this row's [view] carries a comment — `{t}` — and rewriting the table would \
                 drop it. Every comment in a sheet is somebody's reason, so this is refused \
                 rather than tidied. Edit it directly"
            ));
        }
        let key = t.split('=').next().unwrap_or("").trim();
        if !matches!(key, "kind" | "over" | "points" | "y" | "over_x" | "over_y") {
            return Err(format!(
                "this row's [view] has a `{key}`, which this form does not know about. It \
                 rewrites the whole table, so it refuses one it does not understand"
            ));
        }
        if key == "kind" && t.contains("heatmap") {
            return Err(
                "this row draws a heatmap, which has two axes and a grid. This form offers a \
                 number, a line and a bar; turning a heatmap into one of them would throw away \
                 an axis. Edit it directly"
                    .into(),
            );
        }
    }
    let over = over.trim();
    let body = match kind {
        "number" => {
            if !over.is_empty() {
                return Err(
                    "a number is drawn from the row's own answer and is not drawn against \
                     anything. Nothing was written"
                        .into(),
                );
            }
            "kind = \"number\"\n".to_string()
        }
        // The file calls it `y` for a bar. The face says `over` for both,
        // because "the row it is drawn against" is one question.
        "bar" => format!("kind = \"bar\"\ny = {}\n", toml_quote(over)),
        "line" => {
            let n: u32 = match points.trim().parse() {
                Ok(n) if n >= 2 => n,
                Ok(_) => {
                    return Err("a line needs at least two points; one point is a number".into())
                }
                Err(_) => return Err(format!("'{}' is not a number of points", points.trim())),
            };
            if n > 2000 {
                return Err(format!(
                    "{n} points is a run of {n} evaluations of this branch for one picture. The \
                     sweeps in this tree use 60 to 80"
                ));
            }
            format!(
                "kind = \"line\"\nover = {}\npoints = {n}\n",
                toml_quote(over)
            )
        }
        _ => unreachable!("the kind was checked above"),
    };
    let mut out = String::with_capacity(text.len() + 64);
    out.push_str(&text[..from]);
    out.push_str(&body);
    out.push_str(text[to..].trim_start_matches('\n'));
    Ok(out)
}

/// Rewrite how the answer is drawn, and leave the tree consistent or untouched.
///
/// `view_rewrite` decides what the sheet becomes; this is what only the tree can
/// answer. NOTHING ELSE CHECKS IT: the gate validates an input's `var` because
/// an input is an edge, and a view's `over` is not one, so a sweep over a row
/// that was renamed draws nothing and says nothing about why.
pub fn save_view(
    root: &std::path::Path,
    id: &str,
    kind: &str,
    over: &str,
    points: &str,
    base: &str,
) -> Saved {
    let tree = match crate::load::load_all(root) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(format!("the tree does not load: {e}")),
    };
    let Some(sh) = tree.sheets.get(id) else {
        return Saved::Refused(format!("no node '{id}'"));
    };
    let path = sh.dir.join("node.toml");
    let before = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(format!("{}: {e}", path.display())),
    };
    if base != file_hash(&before) {
        return Saved::Stale {
            current: file_hash(&before),
        };
    }
    let over = over.trim();
    if kind != "number" {
        if !tree.sheets.contains_key(over) {
            return Saved::Refused(format!(
                "there is no row '{over}' to draw this against. A sweep over a row that is not \
                 there draws nothing and says nothing about why"
            ));
        }
        if over == id {
            return Saved::Refused(
                "a row cannot be swept against itself — the sweep would hold the answer fixed \
                 and then plot it"
                    .into(),
            );
        }
    }
    let after = match view_rewrite(&before, kind, over, points) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(e),
    };
    commit_edit(root, id, &path, &before, after, false)
}

// ─── publishing a row ────────────────────────────────────────────────────────

/// Why this row cannot be published yet, or nothing.
///
/// A row's `state` is structural and may not be typed into a box: it decides
/// whether anything is generated from the row at all, and a row flipped to
/// `published` half-written generates four files that do not say anything.
/// Moving it is an ACTION with preconditions rather than a field with a value,
/// and this is the list of them.
pub fn unpublishable(sh: &Sheet) -> Vec<String> {
    let mut why = Vec::new();
    if !sh.is_seeded() {
        why.push(format!(
            "this row is already '{}'; publishing moves a seeded row and nothing else",
            sh.state
        ));
        return why;
    }
    let open = unfilled(sh);
    if !open.is_empty() {
        why.push(format!(
            "{} question(s) still block generation: {}",
            open.len(),
            open.join(", ")
        ));
    }
    if !sh.is_declared() {
        // V5, which is an assembly validation and so cannot be seen from the
        // per-node gate this save runs. A computed row with no inputs would
        // publish, pass its own gate, and fail the whole tree's.
        if sh.inputs.is_empty() {
            why.push(
                "a computed row reads something. With no inputs it is a constant written as a \
                 function, and the assembly validations refuse it"
                    .into(),
            );
        }
        if sh.steps.is_empty() {
            why.push(
                "a computed row has at least one algorithm step. With none, the generator has \
                 no hole to put the computation in and emits the row as a declared value of \
                 zero"
                    .into(),
            );
        }
    }
    why
}

/// Move a seeded row to published.
///
/// The one structural field this writes, and only through here. What makes it
/// safe is not that the check list is long but that it is the SAME check the
/// tree runs afterwards: the preconditions above are the assembly validations a
/// per-node gate cannot see, and the save runs the assembly validations too. A
/// row that gets through both is a row that would have got through a terminal.
pub fn publish(root: &std::path::Path, id: &str, base: &str) -> Saved {
    let tree = match crate::load::load_all(root) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(format!("the tree does not load: {e}")),
    };
    let Some(sh) = tree.sheets.get(id) else {
        return Saved::Refused(format!("no node '{id}'"));
    };
    let why = unpublishable(sh);
    if !why.is_empty() {
        return Saved::Refused(format!(
            "this row is not ready to publish — {}. Nothing was written",
            why.join("; ")
        ));
    }
    let path = sh.dir.join("node.toml");
    let before = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(format!("{}: {e}", path.display())),
    };
    if base != file_hash(&before) {
        return Saved::Stale {
            current: file_hash(&before),
        };
    }
    // Textual, like everything else here, and through `assignments` so a
    // `state` inside somebody's paragraph is not mistaken for the key.
    let hits = assignments(&before, "", "state");
    let Some((s0, s1, comment)) = hits.into_iter().next() else {
        return Saved::Refused(
            "this sheet has no `state =` line to move. Every seeded row has one, so this sheet \
             is malformed and not something a form should paper over"
                .into(),
        );
    };
    let mut after = String::with_capacity(before.len() + 16);
    after.push_str(&before[..s0]);
    after.push_str(&format!("state = \"published\"{comment}\n"));
    after.push_str(&before[s1..]);
    // The whole tree, because publishing a row puts it in front of every
    // assembly validation for the first time.
    commit_edit(root, id, &path, &before, after, true)
}
