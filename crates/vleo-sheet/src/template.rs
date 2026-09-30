//! The node form — one self-contained document per node, filled anywhere and
//! applied by a developer.
//!
//! A sheet is edited by people who are not all developers. A payload team knows
//! what their row should say; a customer's engineer knows the number; neither
//! has a checkout. So every node can be downloaded as ONE HTML FILE that needs
//! nothing else: it explains itself, asks the same questions the declaration
//! form asks — read from `form::FIELDS` and `form::ARRAYS`, so the two cannot
//! disagree — shows what the node reads and feeds and what known-good values
//! already hold it, and saves a filled copy of itself. It opens from a disk, an
//! email or a chat. It can be filled by hand or by an assistant, because the
//! content is a plain TOML block inside the file that either can edit.
//!
//! A NEW NODE ARRIVES THE SAME WAY. `xtask form --new` writes a blank form that
//! also asks where the node goes — its parent in the tree and its kind — and
//! lists every row in the tree, so what it reads can be named rather than
//! guessed. Intake checks every interface a form declares: each input must be a
//! row the tree has, carrying the quantity the form says it expects.
//!
//! The filled file goes to the developers. `xtask intake <file>` reads it and
//! says, field by field, what it would change in `node.toml`; `--apply` writes
//! it through the same transaction every other edit takes — regenerate, gate,
//! and put everything back if the gate refuses. The change then goes through
//! git and review like any other, and the next release carries it. The
//! application itself only ever CHECKS a form: a node is changed at a terminal,
//! by a developer, on purpose.
//!
//! THREE VERSIONS, NOT TWO. The file carries the node as it was when the form
//! was downloaded, as well as what the filler made of it. Intake compares both
//! with the node as it is now, so an edit made in the repository in the
//! meantime is never overwritten: a field the form changed that the repository
//! also changed is a CONFLICT, named, and left for a person.
//!
//! AN ASSISTANT MAY NEVER SUPPLY MATHEMATICS, at any face. The form asks whether an
//! assistant helped, and how. Where it helped with the relation, every change
//! to the relation, its steps and its derivation is refused and reported — a
//! developer derives it, or it does not go in. Known-good values are never
//! written at all: a fixture is recorded by a person with its provenance, so
//! the ones a form supplies come out as a request, ready for that person.

use std::collections::BTreeMap;
use std::path::Path;

use crate::form::{self, jq, Blocks, Saved, ARRAYS, FIELDS};
use crate::load::Tree;
use crate::model::{Sheet, View};

/// What a form file declares itself to be. Bumped when the layout of the data
/// block changes, so an old file is recognised rather than misread.
pub const FORMAT: &str = "vleo-node-form/1";

/// The id of the data block a person or an assistant fills.
const DATA_ID: &str = "vleo-node-form";
/// The id of the node as it was when the form was made. Never edited.
const ORIGINAL_ID: &str = "vleo-node-original";

/// Where a known-good value came from. The four a fixture may have: the two it
/// may never have — `self-snapshot`, `agent-generated` — are not offered.
pub const PROVENANCES: &[&str] = &[
    "independent-derivation",
    "published-source",
    "independent-tool",
    "physical-bound",
];

/// How an assistant was involved, as the filler declares it.
pub const AI_HELP: &[&str] = &["none", "wording", "relation"];

/// The arrays that ARE the relation, with the scalar fields that are. An
/// assistant that helped with the relation may not supply any of these.
///
/// The method is the relation once more, and the author's code and cases are
/// the evidence it is checked against: an assistant that wrote any of them has
/// made the check compare the assistant with itself.
const RELATION_FIELDS: &[&str] = &[
    "expression",
    "source",
    "theory_why",
    "theory_reading",
    "method_text",
    "author_code",
    "author_test_code",
];
const RELATION_ARRAYS: &[&str] = &["algorithm", "theory", "case"];
/// The blocks whose change re-confirms the relation itself, and so carries the
/// applying developer's name onto it. The author's cases are refused from an
/// assistant like the relation, but they are the author's evidence, not the
/// relation, and do not move whose name is on it.
const STAMPS_RELATION: &[&str] = &["algorithm", "theory"];

/// The method checker every form carries: `vleo_sheet::method::report_toml`
/// compiled to WebAssembly by `xtask method-wasm`, beside the tree's own files.
pub const CHECKER: &str = "web/method.wasm.gz";

/// The checker, read from the tree when a form is made — never built into a
/// program. A program carrying compressed WebAssembly beside the script that
/// unpacks and runs it is the shape browsers and antivirus block, and the
/// first 0.3.0 Windows kit was blocked by Chrome for exactly that. Missing, the
/// form says so on its check rather than checking by nothing.
fn checker(tree: &Tree) -> Vec<u8> {
    std::fs::read(tree.root.join(CHECKER)).unwrap_or_default()
}

/// Standard base64, for carrying the checker inside the page.
fn base64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut o = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (c[0] as u32) << 16
            | (*c.get(1).unwrap_or(&0) as u32) << 8
            | *c.get(2).unwrap_or(&0) as u32;
        o.push(T[(n >> 18) as usize & 63] as char);
        o.push(T[(n >> 12) as usize & 63] as char);
        o.push(if c.len() > 1 {
            T[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        o.push(if c.len() > 2 {
            T[n as usize & 63] as char
        } else {
            '='
        });
    }
    o
}

// ---------------------------------------------------------------------------
// the content, as the data block holds it

/// One node's content in the shape the form carries it: every field as text,
/// every repeated block as rows of text, and how the answer is drawn.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Content {
    pub fields: BTreeMap<String, String>,
    pub arrays: BTreeMap<String, Vec<BTreeMap<String, String>>>,
    /// `(kind, over, points)`, when the node is drawn in a way this form writes.
    pub view: Option<(String, String, String)>,
}

/// A known-good value the filler supplied. Never written to the sheet.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Known {
    pub label: String,
    /// The inputs it holds at, as the filler wrote them.
    pub inputs: String,
    pub expected: String,
    pub tolerance: String,
    pub provenance: String,
    pub source: String,
}

/// Where a new node goes, as its form asks it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NewNode {
    pub id: String,
    /// The group in the tree it hangs under.
    pub parent: String,
    pub kind: String,
}

/// The kinds a new node may be. Each is built on the shape of an existing row
/// of the same kind, so a kind the tree does not hold yet cannot be asked for.
pub const KINDS: &[&str] = &["computed", "declared", "required", "achieved", "kpi"];

/// Why a node is changing: the de-risking record the form carries, one row of
/// the narrative. See `crate::derisk`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Derisk {
    pub believed: String,
    pub tested: String,
    pub learned: String,
    pub cost: String,
    pub changed: String,
    /// Risk moves, one per line or separated by `;`.
    pub risks: String,
    pub rests_on: String,
    pub breaks_if: String,
}

/// The record's questions, in the order the form asks them: key, question,
/// and why it is asked. One list, read by the page and by intake.
pub const DERISK: &[(&str, &str, &str)] = &[
    (
        "believed",
        "what did we believe",
        "the belief the node rested on before this change — \
      the thing that turned out not to hold, or not to hold well enough",
    ),
    (
        "tested",
        "what did we test",
        "the analysis, comparison, measurement or review that tested \
      it — with where it is written down",
    ),
    (
        "learned",
        "what do we now know",
        "the result: what was wrong or incomplete in the previous \
      version. This is the issue the change fixes",
    ),
    (
        "cost",
        "what did it cost",
        "time, money or effort — so the value of de-risking can be \
      weighed. Optional",
    ),
    (
        "changed",
        "what changes, and what it gains",
        "what this version does differently from the \
      one before, and the benefit — including any change to the plan",
    ),
    (
        "risks",
        "which risks does it open, move or close",
        "one per line: `R-01 L5->L4`, \
      `R-09 closed`, `R-12 opened`. The ids are those registered on the risk-register rows of \
      the management layer. Optional",
    ),
    (
        "rests_on",
        "what does the node rest on now",
        "the belief this version stands on — what is \
      assumed to hold",
    ),
    (
        "breaks_if",
        "what would break that",
        "the observation that would make this version wrong in \
      turn, so the next change is watched for rather than stumbled on",
    ),
];

impl Derisk {
    pub fn get(&self, k: &str) -> &str {
        match k {
            "believed" => &self.believed,
            "tested" => &self.tested,
            "learned" => &self.learned,
            "cost" => &self.cost,
            "changed" => &self.changed,
            "risks" => &self.risks,
            "rests_on" => &self.rests_on,
            "breaks_if" => &self.breaks_if,
            _ => "",
        }
    }
    /// The risk moves, as written.
    pub fn moves(&self) -> Vec<String> {
        self.risks
            .split(['\n', ';'])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    }
    /// What the record is missing: everything on a change; only what the node
    /// rests on and what would break it on a node's first version.
    pub fn missing(&self, first: bool) -> Vec<&'static str> {
        let need: &[&str] = if first {
            &["rests_on", "breaks_if"]
        } else {
            &[
                "believed",
                "tested",
                "learned",
                "changed",
                "rests_on",
                "breaks_if",
            ]
        };
        need.iter()
            .copied()
            .filter(|k| self.get(k).trim().is_empty())
            .collect()
    }
}

/// A filled form, read back.
#[derive(Clone, Debug, Default)]
pub struct Form {
    /// The node it is for; empty on a form for a node that does not exist yet.
    pub node: String,
    /// Set on a form for a new node.
    pub new: Option<NewNode>,
    /// The file hash of `node.toml` the form was made from.
    pub base: String,
    pub name: String,
    pub team: String,
    pub date: String,
    /// `none`, `wording` or `relation`.
    pub ai: String,
    pub notes: String,
    pub original: Content,
    pub filled: Content,
    pub known: Vec<Known>,
    /// Why the node is changing.
    pub derisk: Derisk,
}

/// The fields this form puts to a person for this sheet: asked, and either on
/// the sheet already or ones the form may add.
fn offered(sh: &Sheet) -> Vec<&'static form::Field> {
    let asks = form::asks(sh);
    FIELDS
        .iter()
        .filter(|f| f.asked && asks.iter().any(|a| a.field == f.field && a.available))
        .collect()
}

/// The node as it is now, in the form's shape.
pub fn content(sh: &Sheet) -> Content {
    let mut c = Content::default();
    for f in offered(sh) {
        c.fields
            .insert(f.field.to_string(), form::value(sh, f.field));
    }
    for a in ARRAYS.iter().filter(|a| a.applies(sh)) {
        let rows = form::array_rows(sh, a)
            .into_iter()
            .map(|r| {
                r.into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect::<BTreeMap<_, _>>()
            })
            .collect::<Vec<_>>();
        // A block only some rows carry is written only where it has rows, and
        // read back only where it is written — so absent and empty agree.
        if !a.only_under.is_empty() && rows.is_empty() {
            continue;
        }
        c.arrays.insert(a.name.to_string(), rows);
    }
    c.view = view_of(&sh.view);
    c
}

/// How the answer is drawn, where it is drawn in a way this form writes.
fn view_of(v: &View) -> Option<(String, String, String)> {
    match v {
        View::Number => Some(("number".into(), String::new(), "0".into())),
        View::Line { over, points } => Some(("line".into(), over.clone(), points.to_string())),
        View::Bar { y } => Some(("bar".into(), y.clone(), "0".into())),
        // A heatmap is laid out by hand in the sheet; the form does not offer
        // what it cannot write back.
        View::Heatmap { .. } => None,
    }
}

// ---------------------------------------------------------------------------
// writing TOML — the same layout the page's own writer produces

/// A TOML string. One line where the value is one line; a multi-line basic
/// string where it is not, so a paragraph reads as a paragraph in the file.
/// `</` and `<!` are written as `<` so nothing in a value can close the
/// script element the block sits in.
fn tq(v: &str) -> String {
    let multi = v.contains('\n');
    let mut o = String::with_capacity(v.len() + 8);
    o.push_str(if multi { "\"\"\"\n" } else { "\"" });
    let mut chars = v.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => o.push_str("\\\\"),
            '"' => o.push_str("\\\""),
            '\n' if multi => o.push('\n'),
            '\n' => o.push_str("\\n"),
            '\t' => o.push_str("\\t"),
            '\r' => o.push_str("\\r"),
            '<' if matches!(chars.peek(), Some('/') | Some('!')) => o.push_str("\\u003C"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                o.push_str(&format!("\\u{:04X}", c as u32))
            }
            c => o.push(c),
        }
    }
    o.push_str(if multi { "\"\"\"" } else { "\"" });
    o
}

fn content_toml(c: &Content, o: &mut String) {
    o.push_str("\n[fields]\n");
    for f in FIELDS {
        if let Some(v) = c.fields.get(f.field) {
            o.push_str(&format!("{} = {}\n", f.field, tq(v)));
        }
    }
    if let Some((k, over, pts)) = &c.view {
        o.push_str(&format!(
            "\n[view]\nkind = {}\nover = {}\npoints = {}\n",
            tq(k),
            tq(over),
            tq(pts)
        ));
    }
    for a in ARRAYS {
        for row in c.arrays.get(a.name).into_iter().flatten() {
            o.push_str(&format!("\n[[{}]]\n", a.name));
            for col in a.columns {
                if let Some(v) = row.get(col.key) {
                    o.push_str(&format!("{} = {}\n", col.key, tq(v)));
                }
            }
        }
    }
}

/// The data block of a fresh form: the node as it is, nobody's name yet.
fn data_toml(node: &str, base: &str, c: &Content, new: bool) -> String {
    let mut o = String::new();
    o.push_str(&format!("format = {}\n", tq(FORMAT)));
    o.push_str(&format!("node = {}\nbase = {}\n", tq(node), tq(base)));
    if new {
        o.push_str("\n[new]\nid = \"\"\nparent = \"\"\nkind = \"computed\"\n");
    }
    o.push_str("\n[filled_by]\nname = \"\"\nteam = \"\"\ndate = \"\"\nai = \"none\"\n");
    content_toml(c, &mut o);
    o.push_str("\n[derisk]\n");
    for (k, _, _) in DERISK {
        o.push_str(&format!("{k} = \"\"\n"));
    }
    o.push_str("\n[notes]\ntext = \"\"\n");
    o
}

fn original_toml(node: &str, base: &str, c: &Content, new: bool) -> String {
    let mut o = String::new();
    o.push_str(&format!("format = {}\n", tq(FORMAT)));
    o.push_str(&format!("node = {}\nbase = {}\n", tq(node), tq(base)));
    if new {
        o.push_str("\n[new]\nid = \"\"\nparent = \"\"\nkind = \"computed\"\n");
    }
    content_toml(c, &mut o);
    o
}

/// A new node's content before anybody has said anything: every question
/// blank, drawn as a number.
fn blank() -> Content {
    let mut c = Content::default();
    for f in FIELDS.iter().filter(|f| f.asked) {
        c.fields.insert(f.field.to_string(), String::new());
    }
    for a in ARRAYS {
        c.arrays.insert(a.name.to_string(), Vec::new());
    }
    c.view = Some(("number".into(), String::new(), "0".into()));
    c
}

// ---------------------------------------------------------------------------
// the document

/// Text for HTML, outside a script.
fn he(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// JSON for a script block: `jq`, with `</` kept from closing the element.
fn js(v: &str) -> String {
    jq(v).replace("</", "<\\/").replace("<!", "<\\u0021")
}

fn js_list(items: &[&str]) -> String {
    format!(
        "[{}]",
        items.iter().map(|s| js(s)).collect::<Vec<_>>().join(", ")
    )
}

/// What the page needs besides the content: every question, why it is asked,
/// how its answer is shaped, what the node is, and every row in the tree it
/// could read. Written, never read back. `sh` is `None` on a new node's form.
fn schema(sh: Option<&Sheet>, tree: &Tree) -> String {
    let id = sh.map(|s| s.id.as_str()).unwrap_or("");
    let feeds: Vec<&str> = tree
        .ordered()
        .into_iter()
        .filter(|s| !id.is_empty() && s.inputs.iter().any(|i| i.var == id))
        .map(|s| s.id.as_str())
        .collect();
    let reads: Vec<&str> = sh
        .map(|s| s.inputs.iter().map(|i| i.var.as_str()).collect())
        .unwrap_or_default();
    let mut o = String::from("{\n");
    let get = |f: fn(&Sheet) -> &str| sh.map(f).unwrap_or("");
    for (k, v) in [
        ("node", id),
        ("label", get(|s| &s.label)),
        ("kind", get(|s| &s.kind)),
        ("subsystem", get(|s| &s.subsystem)),
        ("owner", get(|s| &s.owner)),
        ("state", get(|s| &s.state)),
        ("parent", get(|s| &s.parent)),
        ("format", FORMAT),
    ] {
        o.push_str(&format!("  {}: {},\n", js(k), js(v)));
    }
    o.push_str(&format!("  \"new\": {},\n", sh.is_none()));
    o.push_str(&format!("  \"reads\": {},\n", js_list(&reads)));
    o.push_str(&format!("  \"feeds\": {},\n", js_list(&feeds)));
    // EVERY ROW, so an input is chosen from what exists and its quantity is seen
    // before it is wired: the interface is decided on the form, not discovered
    // at intake.
    o.push_str("  \"rows\": [");
    for (i, r) in tree
        .ordered()
        .into_iter()
        .filter(|r| r.state != "deprecated")
        .enumerate()
    {
        o.push_str(&format!(
            "{}\n    [{}, {}, {}, {}]",
            if i > 0 { "," } else { "" },
            js(&r.id),
            js(&r.label),
            js(&r.ty),
            js(&r.unit)
        ));
    }
    o.push_str("\n  ],\n");
    // Where a new node may go: every group, with its layer, in tree order.
    o.push_str("  \"groups\": [");
    let mut groups: Vec<&crate::model::Group> = tree.groups.values().collect();
    groups.sort_by_key(|g| (g.layer, g.order));
    for (i, g) in groups.iter().enumerate() {
        o.push_str(&format!(
            "{}\n    [{}, {}, {}]",
            if i > 0 { "," } else { "" },
            js(&g.id),
            js(&g.label),
            g.layer
        ));
    }
    o.push_str("\n  ],\n");
    // Every work a relation may cite, so a source is picked by its id and the
    // one typed that is not listed yet is seen as such while filling.
    o.push_str("  \"sources\": [");
    for (i, s) in tree
        .sources
        .values()
        .filter(|s| s.status == "current")
        .enumerate()
    {
        o.push_str(&format!(
            "{}\n    [{}, {}, {}]",
            if i > 0 { "," } else { "" },
            js(&s.id),
            js(&s.title),
            js(&s.where_)
        ));
    }
    o.push_str("\n  ],\n");
    o.push_str(&format!("  \"kinds\": {},\n", js_list(KINDS)));
    o.push_str("  \"fields\": [\n");
    let f: Vec<&'static form::Field> = match sh {
        Some(sh) => offered(sh),
        None => FIELDS.iter().filter(|f| f.asked).collect(),
    };
    for (i, x) in f.iter().enumerate() {
        o.push_str(&format!(
            "    {{\"field\": {}, \"group\": {}, \"ask\": {}, \"why\": {}, \"shape\": {}, \
             \"options\": {}, \"required\": {}, \"relation\": {}}}{}\n",
            js(x.field),
            js(x.group),
            js(x.ask),
            js(x.why),
            js(x.shape.name()),
            js_list(x.shape.options()),
            x.blocks,
            RELATION_FIELDS.contains(&x.field),
            if i + 1 == f.len() { "" } else { "," }
        ));
    }
    o.push_str("  ],\n  \"arrays\": [\n");
    let arrays: Vec<&form::Array> = ARRAYS
        .iter()
        .filter(|a| match sh {
            Some(sh) => a.applies(sh),
            None => a.only_under.is_empty(),
        })
        .collect();
    for (i, a) in arrays.iter().enumerate() {
        o.push_str(&format!(
            "    {{\"name\": {}, \"label\": {}, \"why\": {}, \"end_only\": {}, \"relation\": {}, \
             \"columns\": [",
            js(a.name),
            js(a.label),
            js(a.why),
            a.blocks == Blocks::EndOnly,
            RELATION_ARRAYS.contains(&a.name)
        ));
        for (j, c) in a.columns.iter().enumerate() {
            o.push_str(&format!(
                "{}{{\"key\": {}, \"ask\": {}, \"shape\": {}, \"required\": {}, \"managed\": {}}}",
                if j > 0 { ", " } else { "" },
                js(c.key),
                js(c.ask),
                js(c.shape.name()),
                c.required,
                c.managed
            ));
        }
        o.push_str(&format!(
            "]}}{}\n",
            if i + 1 == arrays.len() { "" } else { "," }
        ));
    }
    o.push_str("  ],\n");
    o.push_str(&format!(
        "  \"choices\": {{\"type\": {}, \"unit\": {}}},\n",
        js_list(vleo_units::QUANTITIES),
        js_list(&crate::unit_names())
    ));
    // Each quantity's SI unit, so a case's inputs and answer are asked for in
    // the unit the checker and the generated code work in.
    o.push_str("  \"si\": {");
    for (i, q) in vleo_units::QUANTITIES.iter().enumerate() {
        let sym = vleo_units::quantity_unit(q)
            .map(|u| u.symbol())
            .unwrap_or("");
        o.push_str(&format!(
            "{}{}: {}",
            if i > 0 { ", " } else { "" },
            js(q),
            js(sym)
        ));
    }
    o.push_str("},\n");
    o.push_str(&format!(
        "  \"method\": {{\"version\": {}, \"min_cases\": {}, \"min_refusals\": {}, \"statements\": [{}], \
         \"functions\": [{}], \"constants\": [{}]}},\n",
        crate::method::LANGUAGE_VERSION,
        crate::method::MIN_CASES,
        crate::method::MIN_REFUSALS,
        crate::method::STATEMENTS
            .iter()
            .map(|st| format!("[{}, {}, {}]", js(st.form), js(st.meaning), js(st.example)))
            .collect::<Vec<_>>()
            .join(", "),
        crate::method::FUNCTIONS
            .iter()
            .map(|f| format!("[{}, {}]", js(f.name), js(f.meaning)))
            .collect::<Vec<_>>()
            .join(", "),
        crate::method::KERNEL_CONSTANTS
            .iter()
            .map(|c| format!("[{}, {}, {}]", js(c.name), js(c.unit), js(c.meaning)))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    // THE WORKED EXAMPLE, station by station, from one module — the same one
    // docs/PSEUDOCODE.md and the role guides show.
    o.push_str(&format!(
        "  \"example\": {{\"title\": {}, \"tag\": {}, \"fields\": {{",
        js(crate::example::TITLE),
        js(crate::example::TAG)
    ));
    let mut first = true;
    for fl in FIELDS {
        if let Some(v) = crate::example::field(fl.field) {
            o.push_str(&format!(
                "{}{}: {}",
                if first { "" } else { ", " },
                js(fl.field),
                js(v)
            ));
            first = false;
        }
    }
    o.push_str("}, \"arrays\": {");
    let mut first = true;
    for a in ARRAYS {
        let bl = crate::example::blocks(a.name);
        if bl.is_empty() {
            continue;
        }
        o.push_str(&format!(
            "{}{}: [",
            if first { "" } else { ", " },
            js(a.name)
        ));
        first = false;
        for (i, b) in bl.iter().enumerate() {
            o.push_str(if i > 0 { ", {" } else { "{" });
            for (j, (k, v)) in b.iter().enumerate() {
                o.push_str(&format!(
                    "{}{}: {}",
                    if j > 0 { ", " } else { "" },
                    js(k),
                    js(v)
                ));
            }
            o.push('}');
        }
        o.push(']');
    }
    o.push_str("}},\n");
    o.push_str(&format!(
        "  \"view_kinds\": {},\n  \"provenances\": {},\n  \"ai_help\": {},\n",
        js_list(form::VIEW_KINDS),
        js_list(PROVENANCES),
        js_list(AI_HELP)
    ));
    // WHY IT IS CHANGING: the record's questions, and which kind of decision
    // each field and block is, so the page can say as it is filled which
    // decisions the form moves — and so whether it needs the record.
    o.push_str("  \"derisk\": [");
    for (i, (k, ask, why)) in DERISK.iter().enumerate() {
        o.push_str(&format!(
            "{}\n    [{}, {}, {}]",
            if i > 0 { "," } else { "" },
            js(k),
            js(ask),
            js(why)
        ));
    }
    o.push_str("\n  ],\n  \"about\": {");
    let named: Vec<&str> = FIELDS
        .iter()
        .map(|f| f.field)
        .chain(ARRAYS.iter().map(|a| a.name))
        .chain(["view"])
        .collect();
    let mut first = true;
    for n in named {
        if let Some(a) = crate::derisk::about_of(n) {
            o.push_str(&format!(
                "{}{}: {}",
                if first { "" } else { ", " },
                js(n),
                js(a)
            ));
            first = false;
        }
    }
    o.push_str("},\n");
    o.push_str(&format!(
        "  \"version\": {},\n",
        sh.map(crate::derisk::current).unwrap_or(0)
    ));
    o.push_str("  \"fixtures\": [");
    for (i, fx) in sh
        .map(|s| s.fixtures.as_slice())
        .unwrap_or(&[])
        .iter()
        .enumerate()
    {
        let inputs = fx
            .inputs
            .iter()
            .map(|(k, v)| format!("{k} = {v}"))
            .collect::<Vec<_>>()
            .join(", ");
        o.push_str(&format!(
            "{}\n    {{\"label\": {}, \"inputs\": {}, \"expect\": {}, \"tolerance\": {}, \
             \"provenance\": {}, \"source\": {}}}",
            if i > 0 { "," } else { "" },
            js(&fx.label),
            js(&inputs),
            js(&format!("{}", fx.expect)),
            js(&format!("{}", fx.tolerance)),
            js(&fx.provenance),
            js(&fx.source)
        ));
    }
    o.push_str("\n  ]\n}\n");
    o
}

/// The whole document for one node.
pub fn document(sh: &Sheet, tree: &Tree) -> String {
    let base = std::fs::read_to_string(sh.dir.join("node.toml"))
        .map(|t| form::file_hash(&t))
        .unwrap_or_default();
    let head = format!(
        "<h1>{}</h1>\n<p class=\"nf-id\"><code>{}</code> · {} · {} · owner {} · {}</p>\n",
        he(&sh.label),
        he(&sh.id),
        he(&sh.kind),
        he(&sh.subsystem),
        he(&sh.owner),
        he(&sh.state)
    );
    page(
        &format!("{} — node form", sh.label),
        &head,
        &schema(Some(sh), tree),
        &original_toml(&sh.id, &base, &content(sh), false),
        &data_toml(&sh.id, &base, &content(sh), false),
        &checker(tree),
    )
}

/// A node's form filled with the worked example — FOR THE PIPELINE'S TEST OF
/// THE METHOD PATH, never for a node of the design.
///
/// The example is the circular orbital speed, which is exactly what
/// `orbit_velocity` computes from the same one input, so the pipeline can take
/// that real node through every stage — the form's own check in a browser,
/// intake, the translation, the author's cases, the author's code rerun, the
/// mutation, the interface — on a throwaway checkout, on every pull request.
/// Nothing it writes is ever committed: a method comes from a node's owner.
pub fn document_example(sh: &Sheet, tree: &Tree) -> Result<String, String> {
    let fits = sh.ty == "Velocity"
        && sh.inputs.len() == 1
        && sh.inputs[0].binding == "r"
        && sh.inputs[0].ty == "Length";
    if !fits {
        return Err(format!(
            "the worked example is a Velocity from one Length input `r`; {} is not that node",
            sh.id
        ));
    }
    let base = std::fs::read_to_string(sh.dir.join("node.toml"))
        .map(|t| form::file_hash(&t))
        .unwrap_or_default();
    let original = content(sh);
    let mut c = original.clone();
    for f in FIELDS {
        if f.field.starts_with("method_") || f.field.starts_with("author_") {
            if let Some(v) = crate::example::field(f.field) {
                c.fields.insert(f.field.to_string(), v.to_string());
            }
        }
    }
    for name in ["case", "flight"] {
        let rows = crate::example::blocks(name)
            .into_iter()
            .map(|b| {
                b.into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            })
            .collect();
        c.arrays.insert(name.to_string(), rows);
    }
    let mut data = data_toml(&sh.id, &base, &c, false);
    data = data.replacen(
        "\n[filled_by]\nname = \"\"\nteam = \"\"\ndate = \"\"\nai = \"none\"\n",
        "\n[filled_by]\nname = \"the pipeline, with the worked example\"\nteam = \"\"\ndate = \"\"\nai = \"none\"\n",
        1,
    );
    for (k, v) in [
        (
            "believed",
            "the node's code could only be hand-written holes",
        ),
        (
            "tested",
            "the pipeline's end-to-end run of the method path, with the worked example",
        ),
        (
            "learned",
            "the method, the author's code and the generated code agree on every case",
        ),
        (
            "changed",
            "the node is built from its method — on a throwaway checkout only",
        ),
        ("rests_on", "a two-body, circular orbit"),
        ("breaks_if", "the orbit is noticeably eccentric"),
    ] {
        data = data.replacen(
            &format!("\n{k} = \"\"\n"),
            &format!("\n{k} = {}\n", tq(v)),
            1,
        );
    }
    let head = format!(
        "<h1>{}</h1>\n<p class=\"nf-id\"><code>{}</code> · the worked example, for the pipeline's test</p>\n",
        he(&sh.label),
        he(&sh.id)
    );
    Ok(page(
        &format!("{} — node form (worked example)", sh.label),
        &head,
        &schema(Some(sh), tree),
        &original_toml(&sh.id, &base, &original, false),
        &data,
        &checker(tree),
    ))
}

/// The form for a node that does not exist yet: every question blank, and
/// where it goes asked first.
pub fn document_new(tree: &Tree) -> String {
    let c = blank();
    page(
        "A new node — node form",
        "<h1>A new node</h1>\n<p class=\"nf-id\">a request for a node the design does not have \
         yet — where it goes, what it asks, and what it reads</p>\n",
        &schema(None, tree),
        &original_toml("", "", &c, true),
        &data_toml("", "", &c, true),
        &checker(tree),
    )
}

fn page(
    title: &str,
    head: &str,
    schema: &str,
    original: &str,
    data: &str,
    checker: &[u8],
) -> String {
    let mut o = String::with_capacity(160 * 1024);
    o.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    o.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    o.push_str(&format!("<title>{}</title>\n", he(title)));
    o.push_str("<style>\n");
    o.push_str(PAGE_CSS);
    o.push_str("</style>\n</head>\n<body>\n");
    o.push_str(
        "<header class=\"nf-head\">\n<p class=\"nf-kicker\">VLEO design tool · node form</p>\n",
    );
    o.push_str(head);
    o.push_str(INTRO_HTML);
    o.push_str("</header>\n");
    o.push_str(
        "<noscript><p class=\"nf-warn\">This page fills itself in with JavaScript. Without it, \
         open this file in a text editor and edit the block marked <code>vleo-node-form</code> \
         near the end — it is plain TOML, and every question is asked in the comments above \
         it.</p></noscript>\n",
    );
    o.push_str("<main id=\"nf\"></main>\n");
    o.push_str(&format!(
        "<script type=\"application/json\" id=\"vleo-node-schema\">\n{}</script>\n",
        schema
    ));
    o.push_str(&format!(
        "<!-- The node as it was when this form was made. Do not edit: the developers compare \
         it with the node as it is when the form comes back, so a change made meanwhile is not \
         overwritten. -->\n<script type=\"application/toml\" id=\"{ORIGINAL_ID}\">\n{}</script>\n",
        original
    ));
    o.push_str(&format!(
        "<!-- THE FORM'S CONTENT. This block is what the developers read. Fill it on the page \
         above, or edit it here directly — by hand, or with an assistant: it is TOML, one key \
         per question, and [[input]], [[algorithm]], [[theory]], [[assumption]] and \
         [[known_value]] repeat. Say in [filled_by] ai = \"none\", \"wording\" or \"relation\" \
         how an assistant helped, and in [derisk] why the node is changing: what we believed, \
         what we tested, what we now know, what changes, and what it rests on now. On a new node's form, [new] says where it goes: its id, the \
         group it hangs under, and its kind. -->\n<script type=\"application/toml\" id=\"{DATA_ID}\">\n{}</script>\n",
        data
    ));
    // The method checker, gzipped, as base64. Never edited, and carried into every
    // saved copy, so a filled form still checks its method wherever it goes.
    o.push_str(&format!(
        "<script type=\"application/octet-stream\" id=\"vleo-method-wasm\">{}</script>\n",
        base64(checker)
    ));
    o.push_str("<script>\n");
    o.push_str(PAGE_JS);
    o.push_str("</script>\n</body>\n</html>\n");
    o
}

// ---------------------------------------------------------------------------
// reading a filled form back

/// The text of one `<script … id="…">` block.
fn block<'a>(html: &'a str, id: &str) -> Option<&'a str> {
    let at = html.find(&format!("id=\"{id}\""))?;
    let open = at + html[at..].find('>')? + 1;
    let close = open + html[open..].find("</script>")?;
    Some(&html[open..close])
}

fn text_of(v: Option<&toml::Value>) -> String {
    match v {
        Some(toml::Value::String(s)) => s.clone(),
        // A number typed without quotes — by hand, or by an assistant — is
        // still the number that was meant.
        Some(toml::Value::Float(f)) => format!("{f:?}"),
        Some(toml::Value::Integer(i)) => i.to_string(),
        Some(toml::Value::Boolean(b)) => b.to_string(),
        _ => String::new(),
    }
}

fn content_of(v: &toml::Value) -> Content {
    let mut c = Content::default();
    if let Some(t) = v.get("fields").and_then(|t| t.as_table()) {
        for (k, x) in t {
            c.fields.insert(k.clone(), text_of(Some(x)));
        }
    }
    for a in ARRAYS {
        // A block only some rows carry is absent, not empty, on every other.
        if !a.only_under.is_empty() && v.get(a.name).is_none() {
            continue;
        }
        let rows = v
            .get(a.name)
            .and_then(|x| x.as_array())
            .map(|rows| {
                rows.iter()
                    .map(|r| {
                        a.columns
                            .iter()
                            .map(|col| (col.key.to_string(), text_of(r.get(col.key))))
                            .collect::<BTreeMap<_, _>>()
                    })
                    .collect()
            })
            .unwrap_or_default();
        c.arrays.insert(a.name.to_string(), rows);
    }
    c.view = v.get("view").map(|w| {
        (
            text_of(w.get("kind")),
            text_of(w.get("over")),
            text_of(w.get("points")),
        )
    });
    c
}

/// Read a filled form. Refuses a file that is not one, by what is missing.
pub fn read(html: &str) -> Result<Form, String> {
    let data = block(html, DATA_ID).ok_or(
        "this is not a node form: it has no `vleo-node-form` block. Download the node's form \
         from its page, or with `cargo run -p xtask -- form <node>`",
    )?;
    let original = block(html, ORIGINAL_ID).ok_or(
        "the node as it was when the form was made is missing (`vleo-node-original`), so what \
         the filler changed cannot be told from what the repository changed since. Download a \
         fresh form",
    )?;
    let d: toml::Value = data
        .parse()
        .map_err(|e| format!("the form's content is not valid TOML: {e}"))?;
    let g: toml::Value = original
        .parse()
        .map_err(|e| format!("the form's original block is not valid TOML: {e}"))?;
    let fmt = text_of(d.get("format"));
    if fmt != FORMAT {
        return Err(format!(
            "this form is `{fmt}`, and this tool reads `{FORMAT}`. Download a fresh form"
        ));
    }
    let node = text_of(d.get("node"));
    let new = d.get("new").map(|n| NewNode {
        id: text_of(n.get("id")).trim().to_string(),
        parent: text_of(n.get("parent")).trim().to_string(),
        kind: text_of(n.get("kind")).trim().to_string(),
    });
    if (node.is_empty() && new.is_none()) || text_of(g.get("node")) != node {
        return Err(format!(
            "the form names node '{node}' in its content and '{}' in its original — one of them \
             has been edited, and which node this is for cannot be trusted",
            text_of(g.get("node"))
        ));
    }
    let by = d.get("filled_by");
    let ai = text_of(by.and_then(|b| b.get("ai")));
    let ai = if ai.is_empty() {
        "none".to_string()
    } else {
        ai
    };
    if !AI_HELP.contains(&ai.as_str()) {
        return Err(format!(
            "[filled_by] ai = \"{ai}\" is not one of {}. Say how an assistant helped — the \
             answer decides what may be applied",
            AI_HELP.join(", ")
        ));
    }
    let known = d
        .get("known_value")
        .and_then(|x| x.as_array())
        .map(|rows| {
            rows.iter()
                .map(|r| Known {
                    label: text_of(r.get("label")),
                    inputs: text_of(r.get("inputs")),
                    expected: text_of(r.get("expected")),
                    tolerance: text_of(r.get("tolerance")),
                    provenance: text_of(r.get("provenance")),
                    source: text_of(r.get("source")),
                })
                .filter(|k| !k.expected.trim().is_empty() || !k.label.trim().is_empty())
                .collect()
        })
        .unwrap_or_default();
    let dr = d.get("derisk");
    let dget = |k: &str| text_of(dr.and_then(|x| x.get(k)));
    let derisk = Derisk {
        believed: dget("believed"),
        tested: dget("tested"),
        learned: dget("learned"),
        cost: dget("cost"),
        changed: dget("changed"),
        risks: dget("risks"),
        rests_on: dget("rests_on"),
        breaks_if: dget("breaks_if"),
    };
    Ok(Form {
        derisk,
        base: text_of(g.get("base")),
        name: text_of(by.and_then(|b| b.get("name"))),
        team: text_of(by.and_then(|b| b.get("team"))),
        date: text_of(by.and_then(|b| b.get("date"))),
        ai,
        notes: text_of(d.get("notes").and_then(|n| n.get("text"))),
        original: content_of(&g),
        filled: content_of(&d),
        known,
        node,
        new,
    })
}

// ---------------------------------------------------------------------------
// what a form would change

/// What happens to one change the form asks for.
#[derive(Clone, Debug, PartialEq)]
pub enum Verdict {
    /// Applied by `--apply`.
    Apply,
    /// The node already says this. Nothing to do.
    Already,
    /// The repository changed the same thing since the form was made.
    Conflict(String),
    /// Cannot be applied, and why.
    Refused(String),
}

/// One input a form declares, against the row it names.
#[derive(Clone, Debug, PartialEq)]
pub struct Interface {
    pub binding: String,
    pub var: String,
    /// The quantity the form says it expects.
    pub want: String,
    /// The row's own quantity and unit, when the row exists.
    pub have: String,
    pub unit: String,
    pub label: String,
    /// Why it does not connect, or empty when it does.
    pub why: String,
}

impl Interface {
    pub fn ok(&self) -> bool {
        self.why.is_empty()
    }
}

/// Every input row, checked against the tree: does it name a row, and does
/// that row carry the quantity the form expects?
pub fn interfaces(tree: &Tree, rows: &[BTreeMap<String, String>]) -> Vec<Interface> {
    rows.iter()
        .map(|r| {
            let get = |k: &str| r.get(k).cloned().unwrap_or_default();
            let (binding, var, want) = (get("binding"), get("var"), get("type"));
            let mut i = Interface {
                binding,
                var: var.clone(),
                want: want.clone(),
                have: String::new(),
                unit: String::new(),
                label: String::new(),
                why: String::new(),
            };
            // A row's own answer, or one of the extra variables a row publishes
            // as `<row>.<name>`.
            let found = tree
                .sheets
                .get(&var)
                .map(|s| (s.ty.clone(), s.unit.clone(), s.label.clone()))
                .or_else(|| {
                    let (row, member) = var.split_once('.')?;
                    let s = tree.sheets.get(row)?;
                    s.publishes
                        .iter()
                        .find(|p| p.id == member)
                        .map(|p| (p.ty.clone(), p.unit.clone(), p.label.clone()))
                });
            match found {
                None if var.is_empty() => i.why = "names no row".into(),
                None => {
                    i.why = format!(
                        "there is no row '{var}' in the tree. An input reads a row that exists; a \
                         row that is needed first comes in on its own form"
                    )
                }
                Some((ty, unit, label)) => {
                    i.have = ty.clone();
                    i.unit = unit;
                    i.label = label;
                    if !want.is_empty() && !ty.is_empty() && want != ty {
                        i.why = format!(
                            "'{var}' is a {ty}, and the form expects a {want}. The two sides of an \
                             edge must agree on the quantity, or assembly refuses it"
                        );
                    }
                }
            }
            i
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct Item {
    /// What moves: a field, or a block and its place.
    pub what: String,
    pub from: String,
    pub to: String,
    pub verdict: Verdict,
}

/// Everything a form asks for, and what intake would do with each.
#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub form: Form,
    /// Whether `node.toml` is still the version the form was made from.
    pub base_current: bool,
    /// The hash of `node.toml` this plan was computed against.
    pub current_hash: String,
    pub items: Vec<Item>,
    /// The sheet with every applicable change made, when there is one.
    pub text: Option<String>,
    /// Whether an applied change touches the relation, which carries a name.
    pub relation: bool,
    /// Whether an applied change adds, removes or re-points an input — an edge,
    /// so the whole graph is checked, not only this row.
    pub edges: bool,
    /// Every input the form declares, checked against the tree — when the form
    /// declares or changes any.
    pub interfaces: Vec<Interface>,
    /// Set on a new node's form: where it goes.
    pub new: Option<NewNode>,
    /// What a new node is still missing, or took from the row it was built on
    /// rather than from the form. Reported, for the developer to settle.
    pub open: Vec<String>,
    /// The version this form would record, when it changes what the node
    /// computes and says why.
    pub version: Option<u32>,
    /// The kinds of decision the form's changes move.
    pub about: Vec<&'static str>,
}

impl Plan {
    pub fn applicable(&self) -> usize {
        self.items
            .iter()
            .filter(|i| i.verdict == Verdict::Apply)
            .count()
    }
    pub fn blocked(&self) -> usize {
        self.items
            .iter()
            .filter(|i| matches!(i.verdict, Verdict::Conflict(_) | Verdict::Refused(_)))
            .count()
    }
}

/// Whether a proposed id can be a row's id: lowercase words joined by
/// underscores, starting with a letter, and short enough to read.
pub fn valid_id(id: &str) -> bool {
    let mut c = id.chars();
    matches!(c.next(), Some('a'..='z'))
        && id.len() <= 64
        && id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

/// What a new node's form asks for, checked before anything exists: where it
/// goes, what it is, what it reads. Nothing is built here — `xtask intake
/// --apply` builds the node on the shape of an existing row of the same kind,
/// then applies the form to it with `plan_onto`.
fn plan_new(tree: &Tree, f: Form, n: NewNode) -> Plan {
    let mut p = Plan::default();
    let assisted = f.ai == "relation";
    let place = |what: &str, v: &str, why: Option<String>| Item {
        what: what.into(),
        from: String::new(),
        to: v.into(),
        verdict: why.map(Verdict::Refused).unwrap_or(Verdict::Apply),
    };
    p.items.push(place(
        "new · id",
        &n.id,
        if !valid_id(&n.id) {
            Some("an id is lowercase words joined by underscores, starting with a letter".into())
        } else if tree.sheets.contains_key(&n.id) || tree.groups.contains_key(&n.id) {
            Some(format!("'{}' is already in the tree", n.id))
        } else {
            None
        },
    ));
    p.items.push(place(
        "new · parent",
        &n.parent,
        match tree.groups.get(&n.parent) {
            None => Some(format!(
                "'{}' is not a group in the tree. The form lists every group; a new group is a \
                 developer's decision about the tree's shape",
                n.parent
            )),
            Some(_) => None,
        },
    ));
    p.items.push(place(
        "new · kind",
        &n.kind,
        if !KINDS.contains(&n.kind.as_str()) {
            Some(format!("a node is one of {}", KINDS.join(", ")))
        } else if !tree.sheets.values().any(|s| s.kind == n.kind) {
            Some(format!("no {} row exists to take the shape from", n.kind))
        } else {
            None
        },
    ));
    for fld in FIELDS.iter().filter(|x| x.asked) {
        let v = f.filled.fields.get(fld.field).cloned().unwrap_or_default();
        if v.trim().is_empty() {
            if fld.blocks {
                p.open
                    .push(format!("`{}` is not answered — {}", fld.field, fld.ask));
            }
            continue;
        }
        let verdict = if assisted && RELATION_FIELDS.contains(&fld.field) {
            Verdict::Refused(
                "the form says an assistant helped with the relation; a developer derives it"
                    .into(),
            )
        } else {
            match form::normalise(fld.field, &v) {
                Ok(_) => Verdict::Apply,
                Err(e) => Verdict::Refused(e),
            }
        };
        p.items.push(Item {
            what: fld.field.to_string(),
            from: String::new(),
            to: v,
            verdict,
        });
    }
    for a in ARRAYS {
        for (i, row) in f
            .filled
            .arrays
            .get(a.name)
            .into_iter()
            .flatten()
            .enumerate()
        {
            let missing: Vec<&str> = a
                .columns
                .iter()
                .filter(|c| c.required && !c.managed)
                .filter(|c| row.get(c.key).map(|v| v.trim().is_empty()).unwrap_or(true))
                .map(|c| c.key)
                .collect();
            let verdict = if assisted && RELATION_ARRAYS.contains(&a.name) {
                Verdict::Refused(
                    "an assistant helped with the relation; a developer derives it".into(),
                )
            } else if !missing.is_empty() {
                Verdict::Refused(format!("a {} block needs {}", a.name, missing.join(", ")))
            } else {
                Verdict::Apply
            };
            p.items.push(Item {
                what: format!("{} {} · added", a.name, i + 1),
                from: String::new(),
                to: a
                    .columns
                    .iter()
                    .filter(|c| !c.managed)
                    .filter_map(|c| {
                        row.get(c.key)
                            .filter(|v| !v.is_empty())
                            .map(|v| format!("{} = {}", c.key, short(v)))
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                verdict,
            });
        }
    }
    // A NEW NODE SAYS WHAT IT RESTS ON. Its first version is a belief nobody
    // has tested yet, and the one thing the record needs from the start is what
    // would break it.
    let missing = f.derisk.missing(true);
    p.about = vec!["node"];
    if missing.is_empty() {
        p.version = Some(1);
    }
    p.items.push(Item {
        what: "de-risking · version 1".into(),
        from: String::new(),
        to: short(&f.derisk.rests_on),
        verdict: if missing.is_empty() {
            Verdict::Apply
        } else {
            Verdict::Refused(format!(
                "a new node says what it rests on and what would break it — missing: {}",
                missing.join(", ")
            ))
        },
    });
    if let Some(bad) = risk_problems(tree, &f.derisk) {
        p.items.push(Item {
            what: "de-risking · risks".into(),
            from: String::new(),
            to: f.derisk.risks.clone(),
            verdict: Verdict::Refused(bad),
        });
    }
    let ins = f.filled.arrays.get("input").cloned().unwrap_or_default();
    p.interfaces = interfaces(tree, &ins);
    let bad: Vec<&Interface> = p.interfaces.iter().filter(|i| !i.ok()).collect();
    for it in p
        .items
        .iter_mut()
        .filter(|it| it.what.starts_with("input "))
    {
        if !bad.is_empty() && it.verdict == Verdict::Apply {
            it.verdict = Verdict::Refused("an input does not connect — see the interfaces".into());
        }
    }
    p.new = Some(n);
    p.form = f;
    p
}

/// Plan a new node's form onto the node just built for it: the form's content
/// against the fresh row, as though the form had been made from it.
///
/// The row was built on the shape of `like`, and a blank on the form is not an
/// answer: where the form says nothing and the row holds its model's value — a
/// quantity, a unit, a bound — the value is KEPT, and said, so the developer
/// confirms it rather than finding it later.
pub fn plan_onto(root: &Path, f: &Form, id: &str, like: &str) -> Result<Plan, String> {
    let tree = crate::load::load_all(root).map_err(|e| format!("the tree does not load: {e}"))?;
    let sh = tree
        .sheets
        .get(id)
        .ok_or_else(|| format!("the new node '{id}' is not in the tree"))?;
    let now = content(sh);
    let mut g = f.clone();
    g.node = id.to_string();
    g.new = None;
    g.base = std::fs::read_to_string(sh.dir.join("node.toml"))
        .map(|t| form::file_hash(&t))
        .unwrap_or_default();
    let mut kept = Vec::new();
    for (k, v) in now.fields.iter() {
        let filled = g.filled.fields.entry(k.clone()).or_default();
        if filled.trim().is_empty() && !v.trim().is_empty() {
            *filled = v.clone();
            kept.push(format!(
                "`{k}` was not on the form; it is «{}», taken from {like} — confirm it or change it",
                short(v)
            ));
        }
    }
    // A numbered step keeps the number the new row gives it.
    for a in ARRAYS {
        for (i, row) in g
            .filled
            .arrays
            .entry(a.name.to_string())
            .or_default()
            .iter_mut()
            .enumerate()
        {
            for c in a.columns.iter().filter(|c| c.managed) {
                row.insert(c.key.to_string(), (i + 1).to_string());
            }
        }
    }
    g.filled.view = g.filled.view.clone().or(now.view.clone());
    g.original = now;
    let mut p = plan_form_as(root, g, true)?;
    p.open.extend(kept);
    p.edges = true;
    Ok(p)
}

/// Two field values as the sheet would hold them: a bound of `40` and `40.0`
/// are the same bound.
fn same(field: &str, a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    match (form::normalise(field, a), form::normalise(field, b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    }
}

fn short(v: &str) -> String {
    let one = v.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() > 90 {
        format!("{}…", one.chars().take(89).collect::<String>())
    } else {
        one
    }
}

/// Work out what a filled form would do to its node. Writes nothing.
pub fn plan(root: &Path, html: &str) -> Result<Plan, String> {
    plan_form(root, read(html)?)
}

/// The same, for a form already read.
pub fn plan_form(root: &Path, f: Form) -> Result<Plan, String> {
    plan_form_as(root, f, false)
}

/// Which kind of decision a plan item moves, from what it names: a field, a
/// block and its place, or the view.
fn item_about(what: &str) -> Option<&'static str> {
    let head = what.split([' ', '·']).next().unwrap_or("");
    crate::derisk::about_of(head)
}

/// What is wrong with the risk moves a record names, if anything.
fn risk_problems(tree: &Tree, d: &Derisk) -> Option<String> {
    let known: std::collections::BTreeSet<&str> = tree
        .sheets
        .values()
        .flat_map(|s| s.risks.iter().map(|r| r.id.as_str()))
        .collect();
    let mut bad = Vec::new();
    for m in d.moves() {
        match crate::derisk::parse_move(&m) {
            Err(e) => bad.push(e),
            Ok((id, _)) if !known.contains(id.as_str()) => bad.push(format!(
                "{id} is not registered — a risk is registered first, on a risk-register row's form"
            )),
            Ok(_) => {}
        }
    }
    (!bad.is_empty()).then(|| bad.join("; "))
}

/// Today, as YYYY-MM-DD, from the system clock (vleo_data::clock).
pub fn today() -> String {
    vleo_data::clock::today()
}

/// A form, planned; `first` when it is a new node's first version.
///
/// EVERY CHANGE TO WHAT A NODE COMPUTES SAYS WHICH BELIEF BROKE. A form whose
/// changes move a decision — an input, the output, the model, the maths, the
/// algorithm, how it is drawn — and does not carry a complete de-risking record
/// has those changes WITHHELD, each named, and only its wording goes in. With a
/// complete record, the changes go in and a `[[version]]` is appended to the
/// sheet, numbered, dated, and `next` until a release stamps it.
fn plan_form_as(root: &Path, f: Form, first: bool) -> Result<Plan, String> {
    let tree = crate::load::load_all(root).map_err(|e| format!("the tree does not load: {e}"))?;
    let unlisted = sources_unlisted(&tree, &f);
    let mut p = plan_form_on(&tree, f, first)?;
    p.open.extend(unlisted);
    // WHOSE PLAIN WORDS THESE ARE. A form that changes them names its filler
    // beside them, so the page never presents one person's wording — or an
    // assistant's draft — as another's.
    let wording = p.items.iter().any(|i| {
        i.verdict == Verdict::Apply && i.what.starts_with("explain_") && i.what != "explain_by"
    });
    if let (true, Some(text)) = (wording, p.text.clone()) {
        let who = if p.form.team.trim().is_empty() {
            p.form.name.trim().to_string()
        } else {
            format!("{} ({})", p.form.name.trim(), p.form.team.trim())
        };
        if !who.is_empty() {
            p.text = Some(form::set(&text, "explain_by", &who)?);
        }
    }
    // WHOSE METHOD THIS IS. A method is mathematics: it carries the name of the
    // person who wrote it, the form's filler, and is refused under an
    // assistant's name exactly as a relation is.
    let method = p
        .items
        .iter()
        .any(|i| i.verdict == Verdict::Apply && i.what == "method_text");
    if let (true, Some(text)) = (method, p.text.clone()) {
        let who = p.form.name.trim().to_string();
        form::refuse_agent_attribution(root, &who).map_err(|e| format!("the method: {e}"))?;
        let when = if p.form.date.trim().is_empty() {
            String::new()
        } else {
            format!(" / {}", p.form.date.trim())
        };
        p.text = Some(form::set(&text, "method_by", &format!("{who}{when}"))?);
    }
    Ok(p)
}

/// Every reference the form cites that `sources/` does not list yet. Not a
/// refusal — whoever filled the form names the work, and the developer adds its
/// entry before the row is published — but said at the check, because the gate
/// refuses a published row whose source does not resolve (V8).
fn sources_unlisted(tree: &Tree, f: &Form) -> Vec<String> {
    let mut cited: Vec<(String, String)> = Vec::new();
    let was = f.original.fields.get("source").map(String::as_str);
    if let Some(s) = f.filled.fields.get("source") {
        if Some(s.as_str()) != was || f.new.is_some() {
            cited.push(("the relation".into(), s.trim().to_string()));
        }
    }
    for (i, k) in f.known.iter().enumerate() {
        cited.push((
            format!("known value {}", i + 1),
            k.source.trim().to_string(),
        ));
    }
    cited
        .into_iter()
        .filter(|(_, s)| !s.is_empty() && !tree.sources.contains_key(s))
        .map(|(what, s)| {
            format!(
                "{what} cites '{s}', which is not in sources/sources.toml — add its [[source]] \
                 (id, title, where, status, used_for) before the row is published"
            )
        })
        .collect()
}

fn plan_form_on(tree: &Tree, f: Form, first: bool) -> Result<Plan, String> {
    if let Some(n) = f.new.clone() {
        return Ok(plan_new(tree, f, n));
    }
    let p = plan_edits(tree, f.clone())?;
    let mut about: Vec<&'static str> = p
        .items
        .iter()
        .filter(|i| i.verdict == Verdict::Apply)
        .filter_map(|i| item_about(&i.what))
        .collect();
    // A NEW NODE IS ONE DECISION. Everything on its form differs from the row
    // it was built on, but none of it replaces a belief: the node itself is
    // what is new, and its first version says so.
    if first {
        about = vec!["node"];
    }
    about.sort_by_key(|a| crate::derisk::ABOUT.iter().position(|x| x == a));
    about.dedup();
    if about.is_empty() {
        return Ok(p);
    }
    let missing = f.derisk.missing(first);
    let risk_bad = risk_problems(tree, &f.derisk);
    if missing.is_empty() && risk_bad.is_none() {
        let mut p = p;
        let sh = &tree.sheets[&f.node];
        let n = crate::derisk::current(sh) + 1;
        let applied = |field: &str| {
            p.items
                .iter()
                .any(|i| i.what == field && i.verdict == Verdict::Apply)
        };
        let pick = |field: &str, now: &str| {
            if applied(field) {
                f.filled.fields.get(field).cloned().unwrap_or_default()
            } else {
                now.to_string()
            }
        };
        let v = crate::model::Version {
            n,
            release: crate::derisk::NEXT.into(),
            date: if crate::derisk::date_ok(f.date.trim()) {
                f.date.trim().to_string()
            } else {
                today()
            },
            by: if f.team.trim().is_empty() {
                f.name.trim().to_string()
            } else {
                format!("{} ({})", f.name.trim(), f.team.trim())
            },
            about: about.iter().map(|a| a.to_string()).collect(),
            believed: f.derisk.believed.trim().to_string(),
            tested: f.derisk.tested.trim().to_string(),
            learned: f.derisk.learned.trim().to_string(),
            cost: f.derisk.cost.trim().to_string(),
            changed: if f.derisk.changed.trim().is_empty() && first {
                "first version".into()
            } else {
                f.derisk.changed.trim().to_string()
            },
            risks: f.derisk.moves(),
            rests_on: f.derisk.rests_on.trim().to_string(),
            breaks_if: f.derisk.breaks_if.trim().to_string(),
            relation: pick("expression", &sh.expression),
            source: pick("source", &sh.source),
        };
        let base = p.text.clone().unwrap_or_else(|| {
            std::fs::read_to_string(sh.dir.join("node.toml")).unwrap_or_default()
        });
        p.text = Some(format!("{}{}", base, crate::derisk::version_toml(&v)));
        p.items.push(Item {
            what: format!("de-risking · version {n}"),
            from: String::new(),
            to: format!("{} — {}", about.join(", "), short(&v.changed)),
            verdict: Verdict::Apply,
        });
        p.version = Some(n);
        p.about = about;
        return Ok(p);
    }
    // Withheld: plan again with every decision put back as it was, so only the
    // wording is applied, and name each change that is waiting for its reason.
    let mut g = f.clone();
    for (k, v) in f.original.fields.iter() {
        if crate::derisk::about_of(k).is_some() {
            g.filled.fields.insert(k.clone(), v.clone());
        }
    }
    for a in ARRAYS {
        if crate::derisk::about_of(a.name).is_some() {
            let o = f.original.arrays.get(a.name).cloned().unwrap_or_default();
            g.filled.arrays.insert(a.name.to_string(), o);
        }
    }
    g.filled.view = f.original.view.clone();
    let mut q = plan_edits(tree, g)?;
    let why = match (&risk_bad, missing.is_empty()) {
        (Some(r), true) => r.clone(),
        (r, _) => format!(
            "this form changes {} and does not say which belief broke — the de-risking record \
             is missing: {}{}",
            about.join(", "),
            missing.join(", "),
            r.as_ref().map(|r| format!("; and {r}")).unwrap_or_default()
        ),
    };
    for it in p.items.into_iter().filter(|i| i.verdict == Verdict::Apply) {
        if item_about(&it.what).is_some() {
            q.items.push(Item {
                verdict: Verdict::Refused("withheld until the form says why it changes".into()),
                ..it
            });
        }
    }
    q.items.push(Item {
        what: "de-risking".into(),
        from: String::new(),
        to: String::new(),
        verdict: Verdict::Refused(why),
    });
    q.interfaces = p.interfaces;
    q.about = about;
    q.form = f;
    Ok(q)
}

/// The form's edits to an existing node, without the de-risking record.
fn plan_edits(tree: &Tree, f: Form) -> Result<Plan, String> {
    let sh = tree.sheets.get(&f.node).ok_or_else(|| {
        format!(
            "there is no node '{}' in this tree. A form never adds a node: a new row is a \
             developer's act, with `xtask new`",
            f.node
        )
    })?;
    let before = std::fs::read_to_string(sh.dir.join("node.toml"))
        .map_err(|e| format!("{}: {e}", sh.dir.display()))?;
    let now = content(sh);
    let mut p = Plan {
        current_hash: form::file_hash(&before),
        ..Default::default()
    };
    p.base_current = p.current_hash == f.base;
    let assisted = f.ai == "relation";
    let mut text = before.clone();

    // The scalar fields, in the order the form asks them.
    for fld in FIELDS {
        let (Some(o), Some(n)) = (
            f.original.fields.get(fld.field),
            f.filled.fields.get(fld.field),
        ) else {
            continue;
        };
        if same(fld.field, o, n) {
            continue;
        }
        let c = now.fields.get(fld.field).cloned().unwrap_or_default();
        let verdict = if same(fld.field, &c, n) {
            Verdict::Already
        } else if !same(fld.field, &c, o) {
            Verdict::Conflict(format!(
                "the node has changed this since the form was made; it now says «{}»",
                short(&c)
            ))
        } else if assisted && RELATION_FIELDS.contains(&fld.field) {
            Verdict::Refused(
                "the form says an assistant helped with the relation, and an assistant may never \
                 supply mathematics. A developer derives it, or it does not go in"
                    .into(),
            )
        } else {
            match form::set(&text, fld.field, n) {
                Ok(t) => {
                    text = t;
                    if fld.field == "expression" {
                        p.relation = true;
                    }
                    Verdict::Apply
                }
                Err(e) => Verdict::Refused(e),
            }
        };
        p.items.push(Item {
            what: fld.field.to_string(),
            from: c,
            to: n.clone(),
            verdict,
        });
    }

    // EVERY INPUT CHECKED, WHEN THE INPUTS CHANGE. An input the tree does not
    // have, or of another quantity than the row it names, refuses the input
    // changes by name — here, rather than at the gate as a failed assembly —
    // and everything else on the form still goes in.
    let (o_in, n_in) = (
        f.original.arrays.get("input").cloned().unwrap_or_default(),
        f.filled.arrays.get("input").cloned().unwrap_or_default(),
    );
    let inputs_bad = o_in != n_in && {
        p.interfaces = interfaces(tree, &n_in);
        p.interfaces.iter().any(|i| !i.ok())
    };

    // The repeated blocks: each one as a whole, and then row by row.
    for a in ARRAYS {
        let empty = Vec::new();
        let o = f.original.arrays.get(a.name).unwrap_or(&empty);
        let n = f.filled.arrays.get(a.name).unwrap_or(&empty);
        if o == n {
            continue;
        }
        let c = now.arrays.get(a.name).unwrap_or(&empty);
        let label = |i: usize| format!("{} {}", a.name, i + 1);
        if c == n {
            p.items.push(Item {
                what: a.name.to_string(),
                from: format!("{} block(s)", c.len()),
                to: format!("{} block(s)", n.len()),
                verdict: Verdict::Already,
            });
            continue;
        }
        if c != o {
            p.items.push(Item {
                what: a.name.to_string(),
                from: format!("{} block(s)", c.len()),
                to: format!("{} block(s)", n.len()),
                verdict: Verdict::Conflict(
                    "the node's blocks have changed since the form was made; apply this by hand \
                     against the node as it is"
                        .into(),
                ),
            });
            continue;
        }
        if a.name == "input" && inputs_bad {
            p.items.push(Item {
                what: a.name.to_string(),
                from: format!("{} block(s)", o.len()),
                to: format!("{} block(s)", n.len()),
                verdict: Verdict::Refused(
                    "an input does not connect — see the interfaces. None of the input changes \
                     is applied until every one does"
                        .into(),
                ),
            });
            continue;
        }
        if assisted && RELATION_ARRAYS.contains(&a.name) {
            p.items.push(Item {
                what: a.name.to_string(),
                from: format!("{} block(s)", o.len()),
                to: format!("{} block(s)", n.len()),
                verdict: Verdict::Refused(
                    "the form says an assistant helped with the relation, and these blocks are \
                     the relation. A developer derives them, or they do not go in"
                        .into(),
                ),
            });
            continue;
        }
        // A NUMBERED BLOCK REMOVED OR MOVED IN THE MIDDLE is refused whole. Its
        // number is a hole in the generated code holding somebody's Rust; read
        // position by position, deleting step 2 of 3 would look like step 2
        // rewritten as step 3 and step 3 removed — every body after it silently
        // on the wrong step. The page only offers removing the last; a block
        // edited by hand or by an assistant is held to the same.
        if a.blocks == Blocks::EndOnly {
            if let Some(key) = a.columns.iter().find(|col| col.managed).map(|col| col.key) {
                // A row with no number says nothing about where it was: only a
                // number that differs is a move.
                let moved = (0..o.len().min(n.len())).any(|i| {
                    n[i].get(key).map(|v| !v.is_empty()).unwrap_or(false)
                        && o[i].get(key) != n[i].get(key)
                });
                if moved {
                    p.items.push(Item {
                        what: a.name.to_string(),
                        from: format!("{} block(s)", o.len()),
                        to: format!("{} block(s)", n.len()),
                        verdict: Verdict::Refused(format!(
                            "a numbered {} was removed or moved in the middle. Each number is a \
                             hole in the generated code holding somebody's Rust, so only the \
                             last may be removed and new ones go at the end",
                            a.name
                        )),
                    });
                    continue;
                }
            }
        }
        // Edits in place, then additions at the end, then removals from the
        // end — the only operations that leave a numbered hole where it is.
        let mut refused_rest = false;
        for i in 0..o.len().min(n.len()) {
            for col in a.columns.iter().filter(|col| !col.managed) {
                let ov = o[i].get(col.key).cloned().unwrap_or_default();
                let nv = n[i].get(col.key).cloned().unwrap_or_default();
                if ov == nv {
                    continue;
                }
                let verdict = if refused_rest {
                    Verdict::Refused("an earlier change to this block was refused".into())
                } else {
                    match form::block_text(&text, a.name, "set", i, &[(col.key, nv.clone())]) {
                        Ok(t) => {
                            text = t;
                            Verdict::Apply
                        }
                        Err(e) => {
                            refused_rest = true;
                            Verdict::Refused(e)
                        }
                    }
                };
                if a.name == "input" && verdict == Verdict::Apply {
                    p.edges = true;
                }
                if STAMPS_RELATION.contains(&a.name) && verdict == Verdict::Apply {
                    p.relation = true;
                }
                p.items.push(Item {
                    what: format!("{} · {}", label(i), col.key),
                    from: ov,
                    to: nv,
                    verdict,
                });
            }
        }
        for (i, row) in n.iter().enumerate().skip(o.len()) {
            let vals: Vec<(&str, String)> = a
                .columns
                .iter()
                .filter(|col| !col.managed)
                .map(|col| (col.key, row.get(col.key).cloned().unwrap_or_default()))
                .collect();
            let verdict = if refused_rest {
                Verdict::Refused("an earlier change to this block was refused".into())
            } else {
                match form::block_text(&text, a.name, "add", 0, &vals) {
                    Ok(t) => {
                        text = t;
                        Verdict::Apply
                    }
                    Err(e) => {
                        refused_rest = true;
                        Verdict::Refused(e)
                    }
                }
            };
            if a.name == "input" && verdict == Verdict::Apply {
                p.edges = true;
            }
            if STAMPS_RELATION.contains(&a.name) && verdict == Verdict::Apply {
                p.relation = true;
            }
            p.items.push(Item {
                what: format!("{} · added", label(i)),
                from: String::new(),
                to: vals
                    .iter()
                    .filter(|(_, v)| !v.is_empty())
                    .map(|(k, v)| format!("{k} = {}", short(v)))
                    .collect::<Vec<_>>()
                    .join(", "),
                verdict,
            });
        }
        for i in (n.len()..o.len()).rev() {
            let verdict = if refused_rest {
                Verdict::Refused("an earlier change to this block was refused".into())
            } else {
                match form::block_text(&text, a.name, "remove", i, &[]) {
                    Ok(t) => {
                        text = t;
                        Verdict::Apply
                    }
                    Err(e) => {
                        refused_rest = true;
                        Verdict::Refused(e)
                    }
                }
            };
            if a.name == "input" && verdict == Verdict::Apply {
                p.edges = true;
            }
            if STAMPS_RELATION.contains(&a.name) && verdict == Verdict::Apply {
                p.relation = true;
            }
            p.items.push(Item {
                what: format!("{} · removed", label(i)),
                from: o[i]
                    .iter()
                    .filter(|(_, v)| !v.is_empty())
                    .map(|(k, v)| format!("{k} = {}", short(v)))
                    .collect::<Vec<_>>()
                    .join(", "),
                to: String::new(),
                verdict,
            });
        }
    }

    // How the answer is drawn.
    if let (Some(o), Some(n)) = (&f.original.view, &f.filled.view) {
        if o != n {
            let c = now.view.clone();
            let show = |v: &(String, String, String)| format!("{} {} {}", v.0, v.1, v.2);
            let verdict = if c.as_ref() == Some(n) {
                Verdict::Already
            } else if c.as_ref() != Some(o) {
                Verdict::Conflict("the node's view has changed since the form was made".into())
            } else {
                match form::view_rewrite(&text, &n.0, &n.1, &n.2) {
                    Ok(t) => {
                        text = t;
                        Verdict::Apply
                    }
                    Err(e) => Verdict::Refused(e),
                }
            };
            p.items.push(Item {
                what: "view".into(),
                from: c.as_ref().map(show).unwrap_or_default(),
                to: show(n),
                verdict,
            });
        }
    }

    if p.applicable() > 0 && text != before {
        p.text = Some(text);
    }
    p.form = f;
    Ok(p)
}

/// Apply every change a plan can make, as one edit: write the sheet, regenerate
/// the row, gate it — and put everything back if anything refuses.
pub fn apply(root: &Path, p: &Plan) -> Saved {
    let Some(after) = &p.text else {
        return Saved::Refused("the form asks for nothing that can be applied".into());
    };
    if let Some(why) = form::rustfmt_refusal() {
        return Saved::Refused(why);
    }
    let tree = match crate::load::load_all(root) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(format!("the tree does not load: {e}")),
    };
    let Some(sh) = tree.sheets.get(&p.form.node) else {
        return Saved::Refused(format!("no node '{}'", p.form.node));
    };
    let path = sh.dir.join("node.toml");
    let before = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => return Saved::Refused(format!("{}: {e}", path.display())),
    };
    if form::file_hash(&before) != p.current_hash {
        return Saved::Stale {
            current: form::file_hash(&before),
        };
    }
    // THE RELATION CARRIES THE NAME OF WHOEVER APPLIES IT. The form's filler is
    // recorded in the commit; the sheet names the developer who confirmed the
    // relation by applying it — and refuses if that identity is an assistant's.
    let after = if p.relation {
        let who = match form::git_identity(root) {
            Ok(w) => w,
            Err(e) => return Saved::Refused(e),
        };
        if let Err(e) = form::refuse_agent_attribution(root, &who) {
            return Saved::Refused(e);
        }
        match form::stamp_relation(after, &who) {
            Ok(t) => t,
            Err(e) => return Saved::Refused(e),
        }
    } else {
        after.clone()
    };
    form::commit_edit(root, &p.form.node, &path, &before, after, p.edges)
}

/// The known-good values a form supplied, as `[[fixture]]` blocks for the
/// person who records fixtures. Never written to the sheet by this module: an
/// expected value is recorded by a person, with where it came from.
///
/// IN THE FIXTURE'S UNITS, NOT THE FORM'S. The form asks for the answer in the
/// node's unit and the inputs as a person writes them (`orbit_altitude = 250
/// km`); `fixtures.toml` holds SI, keyed by the node's own binding names. Pasted
/// as typed, 70.2 degrees became 70.2 radians. So each value is converted here
/// and what the form said stays beside it as a comment; whatever cannot be read
/// is left as the comment alone, for the person recording it.
pub fn fixture_request(f: &Form) -> String {
    let unit = f
        .filled
        .fields
        .get("unit")
        .and_then(|u| vleo_units::Unit::from_name(u.trim()));
    let inputs = f.filled.arrays.get("input").cloned().unwrap_or_default();
    let mut o = String::new();
    for k in &f.known {
        o.push_str("[[fixture]]\n");
        o.push_str(&format!("label = {}\n", tq(&k.label)));
        let said = k.expected.trim();
        match (said.parse::<f64>(), unit) {
            (Ok(v), Some(u)) if u.si_factor() != 1.0 => o.push_str(&format!(
                "expect = {}   # the form said {said} {}\n",
                v * u.si_factor(),
                u.symbol()
            )),
            _ => o.push_str(&format!("expect = {said}\n")),
        }
        o.push_str(&format!(
            "tolerance = {}\n",
            if k.tolerance.trim().is_empty() {
                "1e-6"
            } else {
                k.tolerance.trim()
            }
        ));
        o.push_str(&format!("provenance = {}\n", tq(&k.provenance)));
        o.push_str(&format!("source = {}\n", tq(&k.source)));
        match fixture_inputs(&k.inputs, &inputs) {
            Some(t) => o.push_str(&format!(
                "inputs = {{ {t} }}\n# the form gave the inputs as: {}\n\n",
                k.inputs
            )),
            None => o.push_str(&format!(
                "# inputs = {{ … }} in SI, by binding name, each with its unit — the form \
                 gave: {}\n\n",
                k.inputs
            )),
        }
    }
    o
}

/// `orbit_radius = 6778.137 km, …` as `r = 6778137.0, …`: each name is a
/// binding or the row a binding reads, each value a number and a unit symbol.
/// `None` unless every one of them reads, so nothing half-converted is offered.
fn fixture_inputs(said: &str, inputs: &[BTreeMap<String, String>]) -> Option<String> {
    let mut out = Vec::new();
    for part in said
        .split([',', ';'])
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        let (name, rest) = part.split_once('=')?;
        let name = name.trim();
        let (binding, ty) = inputs.iter().find_map(|i| {
            let b = i.get("binding").map(|b| b.trim()).unwrap_or("");
            let v = i.get("var").map(|v| v.trim()).unwrap_or("");
            let t = i.get("type").map(|t| t.trim()).unwrap_or("");
            (name == b || name == v).then(|| (b.to_string(), t))
        })?;
        let mut w = rest.split_whitespace();
        let value: f64 = w.next()?.parse().ok()?;
        let sym = w.collect::<Vec<_>>().join(" ");
        // A bare number is a unit only where there is none to name: a Ratio.
        // Anywhere else `6778` could be metres or kilometres, and is not read.
        let factor = if sym.is_empty() {
            if ty != "Ratio" {
                return None;
            }
            1.0
        } else {
            vleo_units::Unit::NAMES
                .iter()
                .filter_map(|n| vleo_units::Unit::from_name(n))
                .find(|u| u.symbol() == sym)?
                .si_factor()
        };
        out.push(format!("{binding} = {:?}", value * factor));
    }
    (!out.is_empty()).then(|| out.join(", "))
}

// ---------------------------------------------------------------------------
// the page

const INTRO_HTML: &str = r#"<section class="nf-intro">
<p class="nf-answer"><b>Answer first.</b> This file asks for one node of the design to change — or
for a new one — and it is the whole of the request: it needs no connection and nothing installed.
Fill in what you know, say why it is changing, save a copy and send it to the maintainer. They check
it, build it in, and send you a preview of your change to try; when it gives what you expect, you
approve it, and the next release carries it for everyone.</p>
<ol>
<li><b>What is asked.</b> Every question the node answers, in the order it is read: what it is
called and asks, <i>said simply</i>, the relation and where it comes from, the answer and its
bounds, what it reads, and <i>where the simple version breaks</i>. Each question says why it is
asked, and <i>Show the example</i> beside it shows the same question answered for one worked
node. Leave what you cannot answer as it is.</li>
<li><b>The method, your code, your cases.</b> Write the relation once more as a <i>method</i> — a
few lines in a small fixed language — paste the code you wrote and tested, and give at least three
test cases your code answered and one it refuses. The page runs your method on your cases as you
type, with the same checker the maintainer runs: a case that disagrees is shown before you send
anything. Flight software that belongs to this node can be kept here with its test.</li>
<li><b>Why it is changing.</b> A node changes because a belief broke. If your changes move what
the node computes — an input, the output, the model, the maths, the algorithm, how it is drawn —
the section <i>Why it is changing</i> must say what was believed, what was tested, what we now
know and what changes. Without it only your wording is applied. A worked example is in that
section.</li>
<li><b>What is not yours to change here.</b> Where the node sits in the tree, its kind and its
owner are the developers'. If an assistant helped with the <b>relation</b> itself — the equation,
its steps, its derivation, the method, your code or your cases — say so: those are then never
taken from the form. Known values go under <i>known values</i>, with their source.</li>
</ol>
<p class="nf-muted">Tags: <span class="nf-tag req">needed</span> blocks the node until answered ·
<span class="nf-tag rel">the relation</span> never taken from an assistant ·
<span class="nf-tag dr">decision</span> moving it needs <i>why it is changing</i>. To fill it with
an assistant, give it this file and ask it to edit only the block marked
<code>vleo-node-form</code> near the end: plain TOML, one line per answer.</p>
</section>
"#;

const PAGE_CSS: &str = r#":root { --paper:#faf9f5; --card:#ffffff; --ink:#1f1e1b; --ink2:#55524a; --ink3:#8a867b;
  --rule:#dedbd2; --accent:#6b3fa0; --accent-pale:#efe8f7; --warn:#a4262c; --warn-pale:#fbeeee;
  --ok:#2f6b3a; --mono: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
@media (prefers-color-scheme: dark) { :root { --paper:#1b1a18; --card:#23221f; --ink:#ecebe6;
  --ink2:#c4c1b8; --ink3:#8f8b80; --rule:#3a3833; --accent:#b99ae0; --accent-pale:#2e2638;
  --warn:#ef8a8a; --warn-pale:#3a2323; --ok:#8fcf99; } }
* { box-sizing: border-box; }
body { margin: 0; padding: 0 16px 64px; background: var(--paper); color: var(--ink);
  font: 15px/1.5 system-ui, -apple-system, "Segoe UI", sans-serif; }
.nf-head, main { max-width: 920px; margin: 0 auto; }
.nf-kicker { font: 12px var(--mono); letter-spacing: .08em; text-transform: uppercase; color: var(--accent);
  margin: 28px 0 4px; }
h1 { font-size: 26px; margin: 0 0 4px; }
h2 { font-size: 18px; margin: 32px 0 6px; border-bottom: 1px solid var(--rule); padding-bottom: 4px; }
code, .nf-id { font-family: var(--mono); font-size: 13px; }
.nf-id { color: var(--ink2); margin: 0 0 12px; }
.nf-intro { background: var(--card); border: 1px solid var(--rule); border-radius: 6px; padding: 8px 18px; }
.nf-intro li { margin: 4px 0; }
.nf-muted { color: var(--ink3); font-size: 13px; }
.nf-warn { color: var(--warn); background: var(--warn-pale); padding: 10px 14px; border-radius: 6px; }
.nf-bar { position: sticky; top: 0; z-index: 5; background: var(--paper); border-bottom: 1px solid var(--rule);
  padding: 10px 0; display: flex; gap: 8px; flex-wrap: wrap; align-items: center; }
button { font: 13px var(--mono); padding: 6px 12px; border: 1px solid var(--rule); border-radius: 4px;
  background: var(--card); color: var(--ink); cursor: pointer; }
button:hover { border-color: var(--ink3); }
button.nf-primary { background: var(--accent); color: var(--paper); border-color: var(--accent); }
button:disabled { opacity: .45; cursor: default; }
.nf-count { font: 13px var(--mono); color: var(--ink2); }
.nf-q { background: var(--card); border: 1px solid var(--rule); border-radius: 6px; padding: 10px 14px; margin: 10px 0; }
.nf-q.changed { border-color: var(--accent); box-shadow: inset 3px 0 0 var(--accent); }
.nf-q label { display: block; font-weight: 600; }
.nf-why { color: var(--ink3); font-size: 13px; margin: 2px 0 6px; }
.nf-was { color: var(--ink3); font-size: 12px; margin-top: 4px; white-space: pre-wrap; }
.nf-was.nf-bad { color: var(--warn); font-weight: 600; }
.nf-srcbox { display: block; }
.nf-srcbox input { width: 100%; }
.nf-src { margin-top: 4px; }
.nf-src.bad { color: var(--warn); }
.nf-tag { font: 11px var(--mono); padding: 0 6px; border-radius: 3px; margin-left: 6px; vertical-align: 1px;
  border: 1px solid var(--rule); color: var(--ink3); font-weight: 400; }
.nf-tag.req { color: var(--warn); border-color: var(--warn); }
.nf-tag.rel { color: var(--accent); border-color: var(--accent); }
.nf-tag.dr { color: var(--ok); border-color: var(--ok); }
.nf-answer { font-size: 15px; }
.nf-dr { border: 1px solid var(--ok); border-radius: 6px; padding: 4px 14px 10px; margin: 18px 0; }
.nf-dr h2 { border-bottom-color: var(--ok); }
.nf-dr-state { font: 13px var(--mono); padding: 6px 10px; border-radius: 4px; background: var(--accent-pale); }
.nf-dr-state.bad { background: var(--warn-pale); color: var(--warn); }
.nf-ex { font-size: 13px; color: var(--ink2); }
.nf-ex dt { font-weight: 600; margin-top: 4px; }
.nf-ex dd { margin: 0 0 0 12px; }
input, textarea, select { width: 100%; font: 14px var(--mono); color: var(--ink); background: var(--paper);
  border: 1px solid var(--rule); border-radius: 4px; padding: 6px 8px; }
textarea { min-height: 70px; resize: vertical; }
.nf-block { border: 1px dashed var(--rule); border-radius: 6px; padding: 8px 12px; margin: 8px 0; }
.nf-block-h { display: flex; justify-content: space-between; align-items: center; font: 12px var(--mono);
  color: var(--ink3); }
.nf-block .nf-col { margin: 6px 0; }
.nf-block .nf-col span { font-size: 13px; color: var(--ink2); }
.nf-ctx { display: grid; grid-template-columns: max-content 1fr; gap: 4px 14px; font-size: 14px; }
.nf-ctx dt { color: var(--ink3); }
.nf-ctx dd { margin: 0; font-family: var(--mono); font-size: 13px; overflow-wrap: anywhere; }
table.nf-fx { border-collapse: collapse; width: 100%; font-size: 13px; }
table.nf-fx td, table.nf-fx th { border-bottom: 1px solid var(--rule); padding: 4px 6px; text-align: left;
  vertical-align: top; overflow-wrap: anywhere; }
.nf-changes li { margin: 3px 0; }
.nf-grid2 { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
@media (max-width: 640px) { .nf-grid2 { grid-template-columns: 1fr; } .nf-ctx { grid-template-columns: 1fr; } }
textarea.nf-code { white-space: pre; overflow-wrap: normal; overflow-x: auto; tab-size: 2; font-size: 13px;
  min-height: 120px; }
.nf-exbox { margin-top: 6px; }
.nf-exbox summary { cursor: pointer; color: var(--accent); font-size: 13px; }
.nf-exbox pre, .nf-ref pre { background: var(--paper); border: 1px solid var(--rule); border-radius: 4px; padding: 8px 10px;
  overflow-x: auto; font: 12.5px/1.45 var(--mono); white-space: pre; margin: 6px 0; }
.nf-ref { font-size: 13px; }
.nf-ref table { border-collapse: collapse; width: 100%; }
.nf-ref td { border-bottom: 1px solid var(--rule); padding: 3px 6px; vertical-align: top; }
.nf-ins { display: grid; grid-template-columns: max-content 1fr; gap: 4px 10px; align-items: center; }
.nf-ins label { font: 13px var(--mono); font-weight: 400; }
.nf-check { border: 1px solid var(--accent); border-radius: 6px; padding: 8px 14px; margin: 14px 0; background: var(--card); }
.nf-check h3 { margin: 4px 0 6px; font-size: 15px; }
.nf-check ul { margin: 4px 0; padding-left: 20px; }
.nf-check li { margin: 2px 0; font-size: 13.5px; }
.nf-check .ok, .nf-case-res.ok { color: var(--ok); }
.nf-check .bad, .nf-case-res.bad { color: var(--warn); }
.nf-sound { font: 13px var(--mono); padding: 6px 10px; border-radius: 4px; background: var(--accent-pale); }
.nf-sound.bad { background: var(--warn-pale); color: var(--warn); }
@media print { .nf-bar { display: none; } }
"#;

/// The page's behaviour: read the two blocks, draw the form, and save a copy
/// with the data block rewritten. No network, no libraries.
const PAGE_JS: &str = r#"'use strict';
(function () {
  // The page exactly as it arrived, before anything is drawn into it. A saved
  // copy is this with its data block replaced — so the copy is the same form,
  // not a snapshot of whatever the page happened to be showing.
  const PRISTINE = '<!doctype html>\n' + document.documentElement.outerHTML;
  const $ = s => document.querySelector(s);
  const esc = s => String(s == null ? '' : s).replace(/&/g, '&amp;').replace(/</g, '&lt;')
    .replace(/>/g, '&gt;').replace(/"/g, '&quot;');

  // ---- TOML, the subset these blocks use ---------------------------------
  function parseToml(src) {
    const root = {}; let cur = root; let i = 0; const n = src.length;
    const ws = () => { while (i < n && (src[i] === ' ' || src[i] === '\t')) i++; };
    const eol = () => { ws(); if (src[i] === '#') while (i < n && src[i] !== '\n') i++; };
    const fail = m => { throw new Error(m + ' (at character ' + i + ')'); };
    function esc1() {
      const c = src[i++];
      const m = { b: '\b', t: '\t', n: '\n', f: '\f', r: '\r', '"': '"', '\\': '\\' };
      if (c in m) return m[c];
      if (c === 'u' || c === 'U') {
        const len = c === 'u' ? 4 : 8; const h = src.substr(i, len); i += len;
        return String.fromCodePoint(parseInt(h, 16));
      }
      if (c === '\n' || c === ' ' || c === '\t' || c === '\r') { // line-ending backslash
        while (i < n && /[\s]/.test(src[i])) i++; return '';
      }
      fail('bad escape \\' + c);
    }
    function value() {
      if (src.startsWith('"""', i)) {
        i += 3; if (src[i] === '\n') i++; else if (src.startsWith('\r\n', i)) i += 2;
        let o = '';
        while (i < n) {
          if (src.startsWith('"""', i)) {
            let q = 3; while (src[i + q] === '"') q++; // up to two quotes may end the value
            o += '"'.repeat(q - 3); i += q; return o;
          }
          const c = src[i++]; o += c === '\\' ? esc1() : c;
        }
        fail('unclosed """');
      }
      if (src.startsWith("'''", i)) {
        i += 3; if (src[i] === '\n') i++;
        const e = src.indexOf("'''", i); if (e < 0) fail("unclosed '''");
        const o = src.slice(i, e); i = e + 3; return o;
      }
      if (src[i] === '"') {
        i++; let o = '';
        while (i < n && src[i] !== '"') { if (src[i] === '\n') fail('newline in a string'); const c = src[i++]; o += c === '\\' ? esc1() : c; }
        i++; return o;
      }
      if (src[i] === "'") { const e = src.indexOf("'", i + 1); const o = src.slice(i + 1, e); i = e + 1; return o; }
      const m = /^[^\s#,\]}]+/.exec(src.slice(i)); if (!m) fail('a value was expected');
      i += m[0].length; const t = m[0];
      if (t === 'true') return true; if (t === 'false') return false;
      const v = Number(t.replace(/_/g, '')); if (!isFinite(v)) fail('not a value: ' + t); return v;
    }
    function key() {
      if (src[i] === '"') return value();
      const m = /^[A-Za-z0-9_-]+/.exec(src.slice(i)); if (!m) fail('a key was expected');
      i += m[0].length; return m[0];
    }
    function path(close) {
      const parts = []; for (;;) { ws(); parts.push(key()); ws(); if (src[i] === '.') { i++; continue; } break; }
      if (!src.startsWith(close, i)) fail('expected ' + close); i += close.length; return parts;
    }
    while (i < n) {
      ws(); const c = src[i];
      if (c === '\n' || c === '\r') { i++; continue; }
      if (c === '#') { eol(); continue; }
      if (src.startsWith('[[', i)) {
        i += 2; const p = path(']]'); let t = root;
        for (const k of p.slice(0, -1)) t = t[k] = t[k] || {};
        const last = p[p.length - 1]; (t[last] = t[last] || []).push(cur = {}); eol(); continue;
      }
      if (c === '[') {
        i++; const p = path(']'); let t = root; for (const k of p) t = t[k] = t[k] || {}; cur = t; eol(); continue;
      }
      const k = key(); ws(); if (src[i] !== '=') fail('expected = after ' + k); i++; ws();
      cur[k] = value(); eol();
    }
    return root;
  }
  function tq(v) {
    v = String(v == null ? '' : v);
    const multi = v.indexOf('\n') >= 0;
    let o = '';
    for (let k = 0; k < v.length; k++) {
      const c = v[k], code = c.charCodeAt(0);
      if (c === '\\') o += '\\\\';
      else if (c === '"') o += '\\"';
      else if (c === '\n') o += multi ? '\n' : '\\n';
      else if (c === '\t') o += '\\t';
      else if (c === '\r') o += '\\r';
      else if (c === '<' && (v[k + 1] === '/' || v[k + 1] === '!')) o += '\\u003C';
      else if (code < 0x20 || code === 0x7f) o += '\\u' + code.toString(16).toUpperCase().padStart(4, '0');
      else o += c;
    }
    return multi ? '"""\n' + o + '"""' : '"' + o + '"';
  }

  const SCHEMA = JSON.parse($('#vleo-node-schema').textContent);
  const ORIG = parseToml($('#vleo-node-original').textContent);
  let DATA;
  try { DATA = parseToml($('#vleo-node-form').textContent); }
  catch (e) {
    $('#nf').innerHTML = '<p class="nf-warn"><b>The form\'s content block could not be read:</b> ' +
      esc(e.message) + '. It has been edited into something that is not TOML; fix the block ' +
      'marked vleo-node-form in a text editor.</p>';
    return;
  }
  DATA.fields = DATA.fields || {}; DATA.filled_by = DATA.filled_by || {}; DATA.notes = DATA.notes || {};
  DATA.derisk = DATA.derisk || {};
  ORIG.fields = ORIG.fields || {};
  const S = v => (v == null ? '' : String(v));

  function toToml() {
    let o = 'format = ' + tq(SCHEMA.format) + '\nnode = ' + tq(DATA.node) + '\nbase = ' + tq(DATA.base) + '\n';
    if (DATA.new) o += '\n[new]\nid = ' + tq(DATA.new.id) + '\nparent = ' + tq(DATA.new.parent) + '\nkind = ' + tq(DATA.new.kind) + '\n';
    const by = DATA.filled_by;
    o += '\n[filled_by]\nname = ' + tq(by.name) + '\nteam = ' + tq(by.team) + '\ndate = ' + tq(by.date) +
      '\nai = ' + tq(by.ai || 'none') + '\n';
    o += '\n[fields]\n';
    for (const f of SCHEMA.fields) if (f.field in DATA.fields) o += f.field + ' = ' + tq(DATA.fields[f.field]) + '\n';
    if (DATA.view) o += '\n[view]\nkind = ' + tq(DATA.view.kind) + '\nover = ' + tq(DATA.view.over) +
      '\npoints = ' + tq(DATA.view.points) + '\n';
    for (const a of SCHEMA.arrays) for (const r of (DATA[a.name] || [])) {
      o += '\n[[' + a.name + ']]\n';
      for (const c of a.columns) if (c.key in r) o += c.key + ' = ' + tq(r[c.key]) + '\n';
    }
    for (const r of (DATA.known_value || [])) {
      o += '\n[[known_value]]\n';
      for (const k of ['label', 'inputs', 'expected', 'tolerance', 'provenance', 'source']) o += k + ' = ' + tq(r[k]) + '\n';
    }
    o += '\n[derisk]\n';
    for (const [k] of SCHEMA.derisk) o += k + ' = ' + tq(DATA.derisk[k]) + '\n';
    o += '\n[notes]\ntext = ' + tq(DATA.notes.text) + '\n';
    return o;
  }

  // ---- what changed, against the node as the form was made -------------
  function changes() {
    const out = [];
    if (DATA.new && ORIG.new) for (const k of ['id', 'parent', 'kind'])
      if (S(DATA.new[k]) !== S(ORIG.new[k])) out.push('new node ' + k);
    for (const f of SCHEMA.fields) {
      const a = S(ORIG.fields[f.field]), b = S(DATA.fields[f.field]);
      if (a !== b) out.push(f.field);
    }
    for (const a of SCHEMA.arrays) {
      if (JSON.stringify(ORIG[a.name] || []) !== JSON.stringify(DATA[a.name] || [])) out.push(a.label);
    }
    if (ORIG.view && DATA.view && JSON.stringify(ORIG.view) !== JSON.stringify(DATA.view)) out.push('how it is drawn');
    if ((DATA.known_value || []).length) out.push((DATA.known_value || []).length + ' known value(s)');
    if (SCHEMA.derisk.some(([k]) => S(DATA.derisk[k]).trim())) out.push('why it is changing');
    if (S(DATA.notes.text).trim()) out.push('a note for the developers');
    return out;
  }
  // WHICH DECISIONS THE FORM MOVES, as the developers' intake will see them:
  // the same field-to-kind table, sent from the tool.
  function decisions() {
    const kinds = new Set();
    // A new node is one decision, and its first version says so.
    if (DATA.new) return ['node'];
    for (const f of SCHEMA.fields) {
      if (SCHEMA.about[f.field] && S(ORIG.fields[f.field]) !== S(DATA.fields[f.field])) kinds.add(SCHEMA.about[f.field]);
    }
    for (const a of SCHEMA.arrays) {
      if (SCHEMA.about[a.name] && JSON.stringify(ORIG[a.name] || []) !== JSON.stringify(DATA[a.name] || [])) kinds.add(SCHEMA.about[a.name]);
    }
    if (ORIG.view && DATA.view && JSON.stringify(ORIG.view) !== JSON.stringify(DATA.view)) kinds.add('visualisation');
    return [...kinds];
  }
  function recordMissing() {
    const need = DATA.new ? ['rests_on', 'breaks_if'] : ['believed', 'tested', 'learned', 'changed', 'rests_on', 'breaks_if'];
    return need.filter(k => !S(DATA.derisk[k]).trim());
  }
  function paintCount() {
    const c = changes();
    $('#nf-count').textContent = c.length ? c.length + ' change(s): ' + c.join(', ') : 'nothing changed yet';
    const st = $('#nf-dr-state');
    if (!st) return;
    const kinds = decisions(), miss = recordMissing();
    st.className = 'nf-dr-state' + (kinds.length && miss.length ? ' bad' : '');
    st.textContent = !kinds.length
      ? 'Your changes so far are wording only: no record is needed for them.'
      : (DATA.new ? 'A new node. ' : 'Your changes move: ' + kinds.join(', ') + '. ') + (miss.length
        ? 'Still to answer below: ' + miss.map(k => (SCHEMA.derisk.find(d => d[0] === k) || [k, k])[1]).join('; ') +
          (DATA.new ? ' — a new node is not built without them.' : ' — without them the developers apply only your wording.')
        : 'The record is complete: this becomes version ' + (SCHEMA.version + 1) + ' of the node.');
    if (typeof scheduleCheck === 'function') scheduleCheck();
  }

  // ---- controls --------------------------------------------------------
  function control(shape, options, value, onchange, aria) {
    let el;
    const opts = shape === 'quantity' ? SCHEMA.choices.type : shape === 'unit' ? SCHEMA.choices.unit : options;
    if (opts && opts.length) {
      el = document.createElement('select');
      const vals = opts.indexOf(value) >= 0 || value === '' ? opts : [value].concat(opts);
      el.innerHTML = (value === '' ? '<option value=""></option>' : '') +
        vals.map(o => '<option' + (o === value ? ' selected' : '') + '>' + esc(o) + '</option>').join('');
    } else if (shape === 'prose') {
      el = document.createElement('textarea');
      el.value = value; el.rows = Math.min(14, Math.max(3, value.split('\n').length + 1));
    } else if (shape === 'code') {
      // Code: every space kept, no wrapping, and Tab indents instead of
      // leaving the box.
      el = document.createElement('textarea'); el.className = 'nf-code';
      el.value = value; el.spellcheck = false; el.setAttribute('wrap', 'off');
      el.rows = Math.min(24, Math.max(6, value.split('\n').length + 1));
      el.addEventListener('keydown', e => {
        if (e.key !== 'Tab' || e.shiftKey || e.ctrlKey || e.metaKey || e.altKey) return;
        e.preventDefault();
        const a = el.selectionStart, b = el.selectionEnd;
        el.value = el.value.slice(0, a) + '  ' + el.value.slice(b);
        el.selectionStart = el.selectionEnd = a + 2;
        el.dispatchEvent(new Event('input'));
      });
    } else {
      el = document.createElement('input'); el.type = 'text'; el.value = value;
      if (shape === 'number' || shape === 'count') el.inputMode = 'decimal';
      if (shape === 'row') el.setAttribute('list', 'nf-rows');
      if (/(^| )source$/.test(aria || '')) {
        el.setAttribute('list', 'nf-sources');
        const hint = document.createElement('div'); hint.className = 'nf-why nf-src';
        const say = () => {
          const v = el.value.trim(), s = SOURCES.get(v);
          hint.textContent = !v ? 'pick one of the ' + SOURCES.size + ' works in sources/, or name a new one'
            : s ? s[1] + ' — ' + s[2]
            : 'not in sources/ yet: say the full reference here or in the notes, and the developers add its entry before the row is published';
          hint.classList.toggle('bad', !!v && !s);
        };
        el.addEventListener('input', say);
        const box = document.createElement('span'); box.className = 'nf-srcbox';
        box.appendChild(el); box.appendChild(hint); say();
        el.setAttribute('aria-label', aria || '');
        el.addEventListener('input', () => { onchange(el.value); paintCount(); });
        return box;
      }
    }
    el.setAttribute('aria-label', aria || '');
    el.addEventListener(el.tagName === 'SELECT' ? 'change' : 'input', () => { onchange(el.value); paintCount(); });
    return el;
  }

  function question(f) {
    const box = document.createElement('div');
    box.className = 'nf-q'; box.dataset.field = f.field;
    const was = S(ORIG.fields[f.field]);
    box.innerHTML = '<label>' + esc(f.ask.charAt(0).toUpperCase() + f.ask.slice(1)) +
      (f.required ? '<span class="nf-tag req">needed</span>' : '') +
      (f.relation ? '<span class="nf-tag rel">the relation</span>' : '') +
      (SCHEMA.about[f.field] ? '<span class="nf-tag dr" title="changing it needs why it is changing">' +
        esc(SCHEMA.about[f.field]) + '</span>' : '') +
      ' <span class="nf-tag">' + esc(f.field) + '</span></label>' +
      '<div class="nf-why">' + esc(f.why) + '</div>';
    const paint = () => {
      const now = S(DATA.fields[f.field]);
      box.classList.toggle('changed', now !== was);
      let w = box.querySelector('.nf-was');
      if (now !== was) {
        if (!w) { w = document.createElement('div'); w.className = 'nf-was'; box.appendChild(w); }
        w.textContent = 'was: ' + (was || '(blank)');
      } else if (w) w.remove();
    };
    box.appendChild(control(f.shape, f.options, S(DATA.fields[f.field]),
      v => { DATA.fields[f.field] = v; paint(); }, f.field));
    const ex = (SCHEMA.example && SCHEMA.example.fields || {})[f.field];
    if (ex) box.appendChild(exampleBox(ex));
    paint();
    return box;
  }

  // "SHOW THE EXAMPLE": the same question answered for one worked node, so what
  // a good answer looks like is on the page beside the question.
  function exampleBox(text) {
    const d = document.createElement('details'); d.className = 'nf-exbox';
    d.innerHTML = '<summary>Show the example <span class="nf-tag">' + esc(SCHEMA.example.tag) + '</span></summary>' +
      '<pre>' + esc(text) + '</pre>';
    return d;
  }
  function exampleBlocks(a) {
    const bl = (SCHEMA.example && SCHEMA.example.arrays || {})[a.name];
    if (!bl || !bl.length) return null;
    return exampleBox(bl.map((b, i) => a.name + ' ' + (i + 1) + '\n' +
      Object.entries(b).map(([k, v]) => '  ' + k + ': ' + String(v).replace(/\n/g, '\n    ')).join('\n')).join('\n\n'));
  }

  // ---- a case's inputs: one number per input of the node, in SI ----------
  function bindings() {
    return (DATA.input || []).filter(r => S(r.binding).trim()).map(r => [S(r.binding).trim(), S(r.type)]);
  }
  function readInputs(v) {
    const m = {}; const re = /([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([^,}\s]+)/g; let x;
    while ((x = re.exec(S(v)))) m[x[1]] = x[2];
    return m;
  }
  function inputsOk(v) {
    const m = readInputs(v), b = bindings();
    return b.length > 0 && b.every(([k]) => k in m && isFinite(Number(m[k])) && m[k] !== '');
  }
  function inputsEditor(r, onchange) {
    const box = document.createElement('div'); box.className = 'nf-ins';
    const b = bindings();
    if (!b.length) { box.innerHTML = '<span class="nf-muted">add this node\'s inputs first — a case gives each one a value</span>'; return box; }
    const m = readInputs(r.inputs);
    for (const [k, ty] of b) {
      const lab = document.createElement('label'); lab.textContent = k + ' [' + ((SCHEMA.si || {})[ty] || '?') + ']';
      const inp = document.createElement('input'); inp.type = 'text'; inp.inputMode = 'decimal'; inp.value = S(m[k]);
      inp.setAttribute('aria-label', 'case input ' + k);
      inp.addEventListener('input', () => {
        const cur = readInputs(r.inputs); cur[k] = inp.value.trim();
        r.inputs = '{ ' + b.map(([n]) => n).filter(n => S(cur[n]) !== '').map(n => n + ' = ' + cur[n]).join(', ') + ' }';
        inp.style.borderColor = inp.value.trim() && !isFinite(Number(inp.value)) ? 'var(--warn)' : '';
        onchange(); paintCount();
      });
      box.appendChild(lab); box.appendChild(inp);
    }
    return box;
  }

  function blocks(a) {
    const wrap = document.createElement('section');
    const rows = DATA[a.name] = DATA[a.name] || [];
    const draw = () => {
      wrap.innerHTML = '<h2>' + esc(a.label.charAt(0).toUpperCase() + a.label.slice(1)) +
        (a.relation ? ' <span class="nf-tag rel">the relation</span>' : '') + '</h2>' +
        '<p class="nf-why">' + esc(a.why) + '</p>';
      const exb = exampleBlocks(a); if (exb) wrap.appendChild(exb);
      rows.forEach((r, i) => {
        const b = document.createElement('div'); b.className = 'nf-block';
        const last = i === rows.length - 1;
        b.innerHTML = '<div class="nf-block-h"><span>' + esc(a.name) + ' ' + (i + 1) + '</span></div>';
        const rm = document.createElement('button'); rm.type = 'button'; rm.textContent = 'remove';
        rm.disabled = a.end_only && !last;
        rm.title = rm.disabled ? 'Only the last can be removed: each is numbered, and the number is a place in the generated code.' : '';
        rm.onclick = () => { rows.splice(i, 1); draw(); paintCount(); };
        b.firstChild.appendChild(rm);
        const hint = document.createElement('div'); hint.className = 'nf-was';
        // WHAT THIS INPUT CONNECTS TO, as it is typed: the row, its quantity and
        // unit — and a warning when the quantity is not the one expected.
        const paintHint = () => {
          if (a.name !== 'input') return;
          const row = ROWS.get(S(r.var));
          hint.className = 'nf-was' + (row && S(r.type) && row[2] && row[2] !== S(r.type) ? ' nf-bad' : '');
          hint.textContent = !S(r.var) ? 'choose a row from the list' : !row
            ? 'there is no row "' + S(r.var) + '" in the tree — a row that is needed first comes in on its own form'
            : 'reads ' + row[1] + ' — a ' + (row[2] || '?') + ' in ' + (row[3] || '?') +
              (S(r.type) && row[2] && row[2] !== S(r.type) ? ', NOT the ' + S(r.type) + ' this block expects' : '');
        };
        for (const c of a.columns) {
          const col = document.createElement('div'); col.className = 'nf-col';
          if (c.managed) { col.innerHTML = '<span>' + esc(c.key) + ': ' + esc(r[c.key] || '(assigned on apply)') + '</span>'; b.appendChild(col); continue; }
          let ask = c.ask || c.key;
          if (a.name === 'case' && c.key === 'expect') ask = 'the answer your code gave, in ' + ((SCHEMA.si || {})[S(DATA.fields.type) || SCHEMA.type] || 'SI');
          col.innerHTML = '<span>' + esc(ask) + (c.required ? ' <span class="nf-tag req">needed</span>' : '') + '</span>';
          if (a.name === 'case' && (c.key === 'expect' || c.key === 'tolerance')) col.dataset.answer = '1';
          if (c.shape === 'inputs') {
            col.appendChild(inputsEditor(r, () => {}));
          } else {
            const opts = c.shape === 'choice' && a.name === 'case' && c.key === 'refuse' ? ['no', 'yes'] : [];
            col.appendChild(control(c.shape, opts, S(r[c.key]), v => {
              r[c.key] = v;
              if (a.name === 'input' && c.key === 'var' && !S(r.type) && ROWS.get(v)) {
                r.type = ROWS.get(v)[2]; draw(); return;
              }
              if (a.name === 'case' && c.key === 'refuse') showAnswer();
              paintHint();
            }, a.name + ' ' + (i + 1) + ' ' + c.key));
          }
          b.appendChild(col);
        }
        // A case that must be refused has no answer to give.
        const showAnswer = () => {
          if (a.name !== 'case') return;
          for (const el of b.querySelectorAll('[data-answer]')) el.hidden = S(r.refuse) === 'yes';
        };
        showAnswer();
        if (a.name === 'case') {
          const res = document.createElement('div'); res.className = 'nf-was nf-case-res'; res.dataset.case = String(i);
          b.appendChild(res);
        }
        if (a.name === 'input') { b.appendChild(hint); paintHint(); }
        wrap.appendChild(b);
      });
      const add = document.createElement('button'); add.type = 'button';
      add.textContent = 'add ' + (a.end_only ? 'one at the end' : 'one');
      add.onclick = () => {
        const r = {}; for (const c of a.columns) r[c.key] = c.managed ? String(rows.length + 1) : '';
        if (a.name === 'case') { r.refuse = 'no'; r.tolerance = '1e-6'; r.inputs = '{ }'; }
        rows.push(r); draw(); paintCount();
      };
      wrap.appendChild(add);
    };
    draw();
    return wrap;
  }

  function known() {
    const wrap = document.createElement('section');
    const rows = DATA.known_value = DATA.known_value || [];
    const draw = () => {
      wrap.innerHTML = '<h2>Known values</h2><p class="nf-why">An answer you know at given inputs, and where ' +
        'it comes from. A developer records it as a check the node must pass — it is never taken from the ' +
        'tool itself, and never from an assistant.</p>';
      if (SCHEMA.fixtures.length) {
        wrap.insertAdjacentHTML('beforeend', '<p class="nf-muted">Already held by:</p><table class="nf-fx"><tr><th>label</th>' +
          '<th>inputs</th><th>expected</th><th>from</th></tr>' + SCHEMA.fixtures.map(x => '<tr><td>' + esc(x.label) +
          '</td><td>' + esc(x.inputs) + '</td><td>' + esc(x.expect) + ' ± ' + esc(x.tolerance) + '</td><td>' +
          esc(x.provenance) + ' — ' + esc(x.source) + '</td></tr>').join('') + '</table>');
      }
      rows.forEach((r, i) => {
        const b = document.createElement('div'); b.className = 'nf-block';
        b.innerHTML = '<div class="nf-block-h"><span>known value ' + (i + 1) + '</span></div>';
        const rm = document.createElement('button'); rm.type = 'button'; rm.textContent = 'remove';
        rm.onclick = () => { rows.splice(i, 1); draw(); paintCount(); };
        b.firstChild.appendChild(rm);
        const cols = [['label', 'what case this is'], ['inputs', 'the inputs it holds at, each with its unit — e.g. orbit_altitude = 250 km'],
          ['expected', 'the answer, in the node\'s unit'], ['tolerance', 'how close counts, as a fraction (default 1e-6)'],
          ['provenance', 'where it comes from'], ['source', 'the reference — book, paper, page, or who measured it']];
        for (const [k, ask] of cols) {
          const col = document.createElement('div'); col.className = 'nf-col';
          col.innerHTML = '<span>' + esc(ask) + '</span>';
          col.appendChild(control('line', k === 'provenance' ? SCHEMA.provenances : [], S(r[k]), v => { r[k] = v; }, 'known ' + (i + 1) + ' ' + k));
          b.appendChild(col);
        }
        wrap.appendChild(b);
      });
      const add = document.createElement('button'); add.type = 'button'; add.textContent = 'add a known value';
      add.onclick = () => { rows.push({ label: '', inputs: '', expected: '', tolerance: '', provenance: SCHEMA.provenances[0], source: '' }); draw(); paintCount(); };
      wrap.appendChild(add);
    };
    draw();
    return wrap;
  }

  // ---- the page --------------------------------------------------------
  const ROWS = new Map((SCHEMA.rows || []).map(r => [r[0], r]));
  const dl = document.createElement('datalist'); dl.id = 'nf-rows';
  dl.innerHTML = (SCHEMA.rows || []).map(r => '<option value="' + esc(r[0]) + '">' + esc(r[1]) + ' — ' +
    esc(r[2] || '') + ' ' + esc(r[3] || '') + '</option>').join('');
  document.body.appendChild(dl);
  const SOURCES = new Map((SCHEMA.sources || []).map(r => [r[0], r]));
  const ds = document.createElement('datalist'); ds.id = 'nf-sources';
  ds.innerHTML = (SCHEMA.sources || []).map(r => '<option value="' + esc(r[0]) + '">' + esc(r[1]) + '</option>').join('');
  document.body.appendChild(ds);
  // A field that belongs to one kind of node, shown only on that kind.
  const ONLY = { sense: ['required'], declared_value: ['declared', 'required'] };
  const main = $('#nf');
  main.innerHTML = '<div class="nf-bar"><button type="button" class="nf-primary" id="nf-save">save a filled copy</button>' +
    '<button type="button" id="nf-print">print</button><span class="nf-count" id="nf-count"></span></div>';

  const who = document.createElement('section');
  who.innerHTML = '<h2>Who is filling this</h2>';
  const g = document.createElement('div'); g.className = 'nf-grid2';
  for (const [k, ask] of [['name', 'your name'], ['team', 'your team or company'], ['date', 'date (filled in on save if blank)']]) {
    const q = document.createElement('div'); q.className = 'nf-q'; q.innerHTML = '<label>' + esc(ask) + '</label>';
    q.appendChild(control('line', [], S(DATA.filled_by[k]), v => { DATA.filled_by[k] = v; }, k)); g.appendChild(q);
  }
  const ai = document.createElement('div'); ai.className = 'nf-q';
  ai.innerHTML = '<label>did an assistant help?</label><div class="nf-why">none · wording (the text, not the maths) · ' +
    'relation (the equation, its steps or its derivation — those are then derived by a developer, not taken from this form)</div>';
  ai.appendChild(control('choice', SCHEMA.ai_help, S(DATA.filled_by.ai || 'none'), v => { DATA.filled_by.ai = v; }, 'ai'));
  g.appendChild(ai); who.appendChild(g); main.appendChild(who);

  if (SCHEMA.new) {
    DATA.new = DATA.new || { id: '', parent: '', kind: 'computed' };
    const w = document.createElement('section');
    w.innerHTML = '<h2>Where it goes</h2><p class="nf-why">A new node hangs under one group of the tree, in one ' +
      'of its four layers, and is one kind of row. The developers check this first: where a node sits decides ' +
      'who owns it and what it may read.</p>';
    const g3 = document.createElement('div'); g3.className = 'nf-grid2';
    const idq = document.createElement('div'); idq.className = 'nf-q';
    idq.innerHTML = '<label>its id <span class="nf-tag req">needed</span></label><div class="nf-why">lowercase words ' +
      'joined by underscores, starting with a letter — e.g. <code>pay_sensor_mass</code>. It is the answer\'s name ' +
      'everywhere in the design.</div>';
    idq.appendChild(control('line', [], S(DATA.new.id), v => { DATA.new.id = v; }, 'new id')); g3.appendChild(idq);
    const kq = document.createElement('div'); kq.className = 'nf-q';
    kq.innerHTML = '<label>what kind of row <span class="nf-tag req">needed</span></label><div class="nf-why">computed ' +
      '— worked out from what it reads · declared — a number somebody chose · required — a bound the design must ' +
      'meet · achieved — what the design reaches against one · kpi — a figure the programme reports</div>';
    kq.appendChild(control('choice', SCHEMA.kinds, S(DATA.new.kind || 'computed'), v => { DATA.new.kind = v; paintKind(); }, 'new kind'));
    g3.appendChild(kq); w.appendChild(g3);
    const pq = document.createElement('div'); pq.className = 'nf-q';
    pq.innerHTML = '<label>under which group <span class="nf-tag req">needed</span></label><div class="nf-why">every ' +
      'group of the tree, by layer: 1 management · 2 the system · 3 subsystem · 4 the run</div>';
    const sel = document.createElement('select'); sel.setAttribute('aria-label', 'new parent');
    sel.innerHTML = '<option value=""></option>' + SCHEMA.groups.map(g => '<option value="' + esc(g[0]) + '"' +
      (g[0] === S(DATA.new.parent) ? ' selected' : '') + '>Layer ' + g[2] + ' · ' + esc(g[1]) + ' (' + esc(g[0]) + ')</option>').join('');
    sel.addEventListener('change', () => { DATA.new.parent = sel.value; paintCount(); });
    pq.appendChild(sel); w.appendChild(pq);
    main.appendChild(w);
  }
  function paintKind() {
    const kind = DATA.new ? S(DATA.new.kind) : SCHEMA.kind;
    for (const [f, kinds] of Object.entries(ONLY)) {
      const box = document.querySelector('.nf-q[data-field="' + f + '"]');
      if (box) box.hidden = kinds.indexOf(kind) < 0;
    }
  }

  const ctx = document.createElement('section');
  ctx.innerHTML = '<h2>This node</h2><dl class="nf-ctx"><dt>id</dt><dd>' + esc(SCHEMA.node) + '</dd><dt>kind</dt><dd>' +
    esc(SCHEMA.kind) + '</dd><dt>subsystem</dt><dd>' + esc(SCHEMA.subsystem) + ' — owner ' + esc(SCHEMA.owner) +
    '</dd><dt>state</dt><dd>' + esc(SCHEMA.state) + '</dd><dt>reads</dt><dd>' + (esc(SCHEMA.reads.join(', ')) || 'nothing') +
    '</dd><dt>feeds</dt><dd>' + (esc(SCHEMA.feeds.join(', ')) || 'nothing yet') + '</dd></dl>' +
    '<p class="nf-muted">Its place in the tree, its kind and its owner are not on this form: moving a node is a ' +
    'developer\'s decision, taken in the repository.</p>';
  if (!SCHEMA.new) main.appendChild(ctx);

  // THE METHOD, YOUR CODE, YOUR CASES — and the check that runs one against
  // the others while you type, with the same checker intake and the gate use.
  const INTRO = {
    'the method': 'Your relation once more, as a few lines the tool can check, run and translate into the code it ships. ' +
      'It reads the inputs by their names, gives every number its unit, and ends every path with return or refuse.',
    'your code': 'The code you wrote and tested — in MATLAB, Python, C or anything else — and the script that ran it on ' +
      'your test cases. Your cases decide: the method, and the code the tool generates from it, must reproduce every one.'
  };
  function methodRef() {
    const m = SCHEMA.method || {};
    const d = document.createElement('details'); d.className = 'nf-exbox nf-ref';
    d.innerHTML = '<summary>The method language on one page (version ' + esc(m.version) + ')</summary>' +
      '<table>' + (m.statements || []).map(x => '<tr><td><code>' + esc(x[0]) + '</code></td><td>' + esc(x[1]) +
      '<pre>' + esc(x[2]) + '</pre></td></tr>').join('') + '</table>' +
      '<p><b>Functions:</b> ' + (m.functions || []).map(x => '<code title="' + esc(x[1]) + '">' + esc(x[0]) + '</code>').join(' · ') + '</p>' +
      '<p><b>Constants:</b> ' + (m.constants || []).map(x => '<code title="' + esc(x[2]) + '">' + esc(x[0]) + '</code> [' + esc(x[1]) + ']').join(' · ') + '</p>' +
      '<p class="nf-muted">Units go in brackets straight after a number: <code>250 [km]</code>, <code>30 [deg]</code>, ' +
      '<code>3.986e14 [m^3/s^2]</code>. A bare 0 is zero of anything. The full reference is docs/PSEUDOCODE.md.</p>';
    return d;
  }
  const check = document.createElement('section'); check.className = 'nf-check'; check.id = 'nf-check';
  let placedCases = false, placedInputs = false;
  const groups = [];
  for (const f of SCHEMA.fields) if (groups.indexOf(f.group) < 0) groups.push(f.group);
  for (const grp of groups) {
    // What the node reads comes before the method, which reads it by name.
    if (grp === 'the method') {
      const ia = SCHEMA.arrays.find(a => a.name === 'input');
      if (ia) { main.appendChild(blocks(ia)); placedInputs = true; }
    }
    const sec = document.createElement('section');
    sec.innerHTML = '<h2>' + esc(grp.charAt(0).toUpperCase() + grp.slice(1)) + '</h2>' +
      (INTRO[grp] ? '<p class="nf-why">' + esc(INTRO[grp]) + '</p>' : '');
    if (grp === 'the method') sec.appendChild(methodRef());
    for (const f of SCHEMA.fields.filter(x => x.group === grp)) sec.appendChild(question(f));
    main.appendChild(sec);
    if (grp === 'your code') {
      const ca = SCHEMA.arrays.find(a => a.name === 'case');
      if (ca) { main.appendChild(blocks(ca)); main.appendChild(check); placedCases = true; }
    }
  }
  for (const a of SCHEMA.arrays) {
    if (a.name === 'flight' || (a.name === 'case' && placedCases) || (a.name === 'input' && placedInputs)) continue;
    main.appendChild(blocks(a));
    if (a.name === 'case') main.appendChild(check);
  }
  for (const a of SCHEMA.arrays) if (a.name === 'flight') main.appendChild(blocks(a));

  // ---- the check: the method run on your cases, in this page -------------
  let VM = null, VMerr = '';
  async function vm() {
    if (VM || VMerr) return VM;
    try {
      const bin = atob((($('#vleo-method-wasm') || {}).textContent || '').trim());
      if (!bin) throw new Error('this form was made without its checker, web/method.wasm.gz');
      const gz = new Uint8Array(bin.length);
      for (let k = 0; k < bin.length; k++) gz[k] = bin.charCodeAt(k);
      // Carried compressed, to keep the form small; unpacked by the browser.
      if (typeof DecompressionStream !== 'function') throw new Error('this browser is too old to unpack the checker');
      const bytes = await new Response(new Blob([gz]).stream().pipeThrough(new DecompressionStream('gzip'))).arrayBuffer();
      VM = (await WebAssembly.instantiate(bytes, {})).instance.exports;
    } catch (e) { VMerr = String(e && e.message || e); }
    return VM;
  }
  async function runReport(text) {
    const v = await vm(); if (!v) return null;
    const enc = new TextEncoder().encode(text);
    const p = v.vleo_alloc(enc.length);
    new Uint8Array(v.memory.buffer, p, enc.length).set(enc);
    const out = v.vleo_report(p, enc.length), n = v.vleo_report_len();
    return JSON.parse(new TextDecoder().decode(new Uint8Array(v.memory.buffer, out, n)));
  }
  const num = v => { const x = Number(S(v).trim()); return isFinite(x) && S(v).trim() !== '' ? String(x) : null; };
  // Only a case whose answer YOU have typed is run: the method's value is shown
  // beside yours, never before it, so the tool can never be where your answer
  // came from.
  function checkToml() {
    // The plain form `method::report_plain` reads — see its doc comment.
    const one = v => S(v).replace(/[\r\n]+/g, ' ').trim();
    let o = 'output ' + one(DATA.fields.type) + '\n';
    for (const [k, ty] of bindings()) o += 'input ' + one(k) + ' ' + one(ty) + '\n';
    const used = [], waiting = [];
    (DATA.case || []).forEach((r, i) => {
      const refuse = S(r.refuse) === 'yes';
      if ((!refuse && num(r.expect) == null) || !inputsOk(r.inputs)) { waiting.push(i); return; }
      const m = readInputs(r.inputs);
      const ins = bindings().map(([k]) => k + '=' + Number(m[k])).join(';');
      o += 'case ' + (refuse ? '1 - - ' : '0 ' + num(r.expect) + ' ' + (num(r.tolerance) || '0') + ' ') +
        ins + ' ' + (one(r.label) || 'case ' + (i + 1)) + '\n';
      used.push(i);
    });
    o += 'method\n' + S(DATA.fields.method_text);
    return { text: o, used, waiting };
  }
  let timer = null, seq = 0;
  function scheduleCheck() { clearTimeout(timer); timer = setTimeout(doCheck, 350); }
  async function doCheck() {
    const my = ++seq;
    const head = '<h3>Check: the method against your cases</h3>';
    const resEls = [...document.querySelectorAll('.nf-case-res')];
    for (const el of resEls) { el.textContent = ''; el.className = 'nf-was nf-case-res'; }
    if (!S(DATA.fields.method_text).trim()) {
      check.innerHTML = head + '<p class="nf-why">Write the method above, and your cases, and this runs one on the ' +
        'other as you type — the same check the developers run when the form arrives.</p>';
      return;
    }
    const { text, used, waiting } = checkToml();
    const r = await runReport(text);
    if (my !== seq) return;
    if (!r) {
      check.innerHTML = head + '<p class="nf-warn">This browser could not start the checker (' + esc(VMerr) +
        '). The developers run the same check when the form arrives; nothing is lost.</p>';
      return;
    }
    let h = head;
    if (r.error) h += '<p class="bad">' + esc(r.error) + '</p>';
    if (r.diags.length) h += '<ul>' + r.diags.map(d => '<li class="' + (d.severity === 'error' ? 'bad' : '') + '">' +
      (d.line ? 'line ' + d.line + ': ' : '') + (d.severity === 'note' ? 'note: ' : '') + esc(d.msg) + '</li>').join('') + '</ul>';
    else if (!r.error) h += '<p class="ok">The method reads, and its units agree.</p>';
    const agree = r.cases.filter(c => c.agrees).length;
    if (r.cases.length) h += '<p>' + agree + ' of ' + r.cases.length + ' case(s) agree with your code.</p><ul>' +
      r.cases.map(c => '<li class="' + (c.agrees ? 'ok' : 'bad') + '">' + esc(c.label) + ': ' + esc(c.text) +
        (c.got != null && c.agrees && !c.refuse ? ' — the method gives ' + esc(String(Number(c.got))) : '') + '</li>').join('') + '</ul>';
    if (waiting.length) h += '<p class="nf-muted">' + waiting.length + ' case(s) not run yet: give every input a number, and ' +
      'type the answer your code gave (or say it must be refused). The method\'s value is shown only after yours.</p>';
    if (r.shortfall.length) h += '<ul>' + r.shortfall.map(x => '<li class="bad">' + esc(x) + '</li>').join('') + '</ul>';
    h += '<p class="nf-sound' + (r.sound ? '' : ' bad') + '">' + (r.sound
      ? 'Sound: the method checks, and agrees with every one of your cases.'
      : 'Not sound yet — the developers will see exactly what is shown here.') + '</p>';
    check.innerHTML = h;
    used.forEach((i, k) => {
      const c = r.cases[k], el = resEls.find(e => e.dataset.case === String(i));
      if (!c || !el) return;
      el.className = 'nf-was nf-case-res ' + (c.agrees ? 'ok' : 'bad');
      el.textContent = (c.agrees ? '✓ ' : '✗ ') + c.text +
        (c.got != null && c.agrees && !c.refuse ? ' — the method gives ' + String(Number(c.got)) : '');
    });
    for (const i of waiting) {
      const el = resEls.find(e => e.dataset.case === String(i));
      if (el) el.textContent = 'not run yet: every input needs a number, and your code\'s answer (or refuse = yes)';
    }
  }

  if (DATA.view) {
    const v = document.createElement('section');
    v.innerHTML = '<h2>How the answer is drawn</h2><p class="nf-why">A number, a line over one of its inputs, or bars.</p>';
    const g2 = document.createElement('div'); g2.className = 'nf-grid2';
    for (const [k, ask, sh, opts] of [['kind', 'drawn as', 'choice', SCHEMA.view_kinds], ['over', 'over which input (a line), or which row (bars)', 'line', []],
      ['points', 'how many points along the line', 'count', []]]) {
      const q = document.createElement('div'); q.className = 'nf-q'; q.innerHTML = '<label>' + esc(ask) + '</label>';
      q.appendChild(control(sh, opts, S(DATA.view[k]), val => { DATA.view[k] = val; }, 'view ' + k)); g2.appendChild(q);
    }
    v.appendChild(g2); main.appendChild(v);
  }
  // ---- why it is changing: the de-risking record ----------------------
  const dr = document.createElement('section'); dr.className = 'nf-dr';
  dr.innerHTML = '<h2>Why it is changing</h2><p class="nf-why">' + (SCHEMA.new
    ? 'A new node is a belief nobody has tested yet. Say what it rests on and what would break it — the ' +
      'rest can wait for the first time it changes.'
    : 'A node changes because a belief broke: somebody tested it and learned otherwise. The answers here ' +
      'become version ' + (SCHEMA.version + 1) + ' of this node, and one row of the programme\'s ' +
      'de-risking narrative. Wording alone needs none of it.') + '</p><p class="nf-dr-state" id="nf-dr-state"></p>';
  for (const [k, ask, why] of SCHEMA.derisk) {
    if (SCHEMA.new && ['believed', 'tested', 'learned', 'cost'].indexOf(k) >= 0) continue;
    const q = document.createElement('div'); q.className = 'nf-q'; q.dataset.derisk = k;
    q.innerHTML = '<label>' + esc(ask.charAt(0).toUpperCase() + ask.slice(1)) + '</label><div class="nf-why">' + esc(why) + '</div>';
    q.appendChild(control(k === 'cost' ? 'line' : 'prose', [], S(DATA.derisk[k]), v => { DATA.derisk[k] = v; }, 'why ' + k));
    dr.appendChild(q);
  }
  dr.insertAdjacentHTML('beforeend', '<details class="nf-ex"><summary>A worked example ' +
    '<span class="nf-tag">illustrative — not this node</span></summary><dl>' +
    '<dt>What we believed</dt><dd>That the drag coefficient could be bounded from modelling alone, without ' +
    'reconciliation against flight decay data.</dd>' +
    '<dt>What we tested</dt><dd>Free-molecular gas-surface interaction runs against two density models, ' +
    'cross-checked against published on-orbit decay in the 350–380 km band.</dd>' +
    '<dt>What we now know</dt><dd>The 1-sigma band narrowed from 28% to 19%, but the residual is dominated by ' +
    'energy accommodation, which only flight data will settle. Ground work alone will not reach the 15% target.</dd>' +
    '<dt>What it cost</dt><dd>$310k</dd>' +
    '<dt>What changes, and what it gains</dt><dd>Propellant margin held at 30% to CDR rather than released; the ' +
    'air-breathing propulsion decision moved to Q2-27.</dd>' +
    '<dt>Risks</dt><dd>R-09 closed · R-01 L5-&gt;L4</dd>' +
    '<dt>Rests on now</dt><dd>The 19% band, until flight decay data exists.</dd>' +
    '<dt>Would break if</dt><dd>The first flight decay residual falls outside the 19% band.</dd></dl></details>');
  main.appendChild(dr);

  main.appendChild(known());

  const notes = document.createElement('section');
  notes.innerHTML = '<h2>Anything else the developers should know</h2><p class="nf-why">What you changed and why, what ' +
    'you were unsure of, what you would like to see when you run it.</p>';
  const nq = document.createElement('div'); nq.className = 'nf-q';
  nq.appendChild(control('prose', [], S(DATA.notes.text), v => { DATA.notes.text = v; }, 'notes'));
  notes.appendChild(nq); main.appendChild(notes);

  paintKind();
  $('#nf-print').onclick = () => window.print();
  $('#nf-save').onclick = () => {
    if (!S(DATA.filled_by.date)) DATA.filled_by.date = new Date().toISOString().slice(0, 10);
    const open = '<script type="application/toml" id="vleo-node-form">';
    const at = PRISTINE.indexOf(open);
    const end = PRISTINE.indexOf('</' + 'script>', at);
    const html = PRISTINE.slice(0, at + open.length) + '\n' + toToml() + PRISTINE.slice(end);
    const a = document.createElement('a');
    a.href = URL.createObjectURL(new Blob([html], { type: 'text/html' }));
    a.download = (SCHEMA.new ? 'new-' + (S(DATA.new.id) || 'node') : SCHEMA.node) + '.node-form.html';
    document.body.appendChild(a); a.click(); a.remove();
    setTimeout(() => URL.revokeObjectURL(a.href), 4000);
  };
  paintCount();
  doCheck();
})();
"#;

#[cfg(test)]
mod tests {
    use super::tq;

    /// Every string the writer can meet comes back from a TOML parser as it
    /// went in, and nothing it writes can close the script element it sits in.
    #[test]
    fn a_written_string_reads_back_exactly_and_never_closes_its_script() {
        for v in [
            "",
            "plain",
            "a </script> and <!-- a comment",
            "quote \" and \"\"\" triple",
            "ends with a quote\"",
            "two lines\nthe second ending in a quote\"",
            "back\\slash and tab\there",
            "\u{1}control and \u{7f}delete",
            "ünïcødé ✓ — and ≤ ≥",
        ] {
            let text = format!("k = {}\n", tq(v));
            assert!(!text.to_ascii_lowercase().contains("</script"), "{text}");
            assert!(!text.contains("<!"), "{text}");
            let back: toml::Value = text.parse().unwrap_or_else(|e| panic!("{text}: {e}"));
            assert_eq!(back.get("k").and_then(|x| x.as_str()), Some(v), "{text}");
        }
    }
}
