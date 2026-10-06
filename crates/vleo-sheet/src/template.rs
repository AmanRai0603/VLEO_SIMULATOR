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

use crate::{Error, ErrorKind};
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
/// The method is the relation once more, and the node engineer's code and cases are
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
/// applying developer's name onto it. The node engineer's cases are refused from an
/// assistant like the relation, but they are the node engineer's evidence, not the
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
pub(crate) fn checker(tree: &Tree) -> Vec<u8> {
    std::fs::read(tree.root.join(CHECKER)).unwrap_or_default()
}

/// Standard base64, for carrying the checker inside the page.
pub fn base64(bytes: &[u8]) -> String {
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
use crate::text::html as he;

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
    let head = f(
        "heading-node",
        &[
            ("label", &he(&sh.label)),
            ("id", &he(&sh.id)),
            ("kind", &he(&sh.kind)),
            ("subsystem", &he(&sh.subsystem)),
            ("owner", &he(&sh.owner)),
            ("state", &he(&sh.state)),
        ],
    );
    page(
        &f("title-node", &[("label", &sh.label)]),
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
/// intake, the translation, the node engineer's cases, the node engineer's code rerun, the
/// mutation, the interface — on a throwaway checkout, on every pull request.
/// Nothing it writes is ever committed: a method comes from a node's owner.
pub fn document_example(sh: &Sheet, tree: &Tree) -> Result<String, Error> {
    let fits = sh.ty == "Velocity"
        && sh.inputs.len() == 1
        && sh.inputs[0].binding == "r"
        && sh.inputs[0].ty == "Length";
    if !fits {
        return Err(Error::new(
            ErrorKind::Malformed,
            format!(
                "the worked example is a Velocity from one Length input `r`; {} is not that node",
                sh.id
            ),
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
            "the method, the node engineer's code and the generated code agree on every case",
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
    let head = f(
        "heading-example",
        &[("label", &he(&sh.label)), ("id", &he(&sh.id))],
    );
    Ok(page(
        &f("title-example", &[("label", &sh.label)]),
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
        t("title-new"),
        t("heading-new"),
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
    // The method checker, gzipped, as base64. Never edited, and carried into every
    // saved copy, so a filled form still checks its method wherever it goes.
    let o = f(
        "body",
        &[
            ("heading", head),
            ("schema", schema),
            ("original", original),
            ("data", data),
            ("checker", &base64(checker)),
            ("script", PAGE_JS),
        ],
    );
    crate::shell::fill(&crate::shell::Page {
        title,
        head: &f("head", &[("css", PAGE_CSS)]),
        body: &o,
        ..Default::default()
    })
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
pub fn read(html: &str) -> Result<Form, Error> {
    let data = block(html, DATA_ID).ok_or_else(|| {
        Error::new(
            ErrorKind::Malformed,
            "this is not a node form: it has no `vleo-node-form` block. Download the node's form \
         from its page, or with `cargo run -p xtask -- form <node>`",
        )
    })?;
    let original = block(html, ORIGINAL_ID).ok_or_else(|| {
        Error::new(
            ErrorKind::Malformed,
            "the node as it was when the form was made is missing (`vleo-node-original`), so what \
         the filler changed cannot be told from what the repository changed since. Download a \
         fresh form",
        )
    })?;
    let d: toml::Value = data.parse().map_err(|e| {
        Error::new(
            ErrorKind::Malformed,
            format!("the form's content is not valid TOML: {e}"),
        )
    })?;
    let g: toml::Value = original.parse().map_err(|e| {
        Error::new(
            ErrorKind::Malformed,
            format!("the form's original block is not valid TOML: {e}"),
        )
    })?;
    let fmt = text_of(d.get("format"));
    if fmt != FORMAT {
        return Err(Error::new(
            ErrorKind::Malformed,
            format!("this form is `{fmt}`, and this tool reads `{FORMAT}`. Download a fresh form"),
        ));
    }
    let node = text_of(d.get("node"));
    let new = d.get("new").map(|n| NewNode {
        id: text_of(n.get("id")).trim().to_string(),
        parent: text_of(n.get("parent")).trim().to_string(),
        kind: text_of(n.get("kind")).trim().to_string(),
    });
    if (node.is_empty() && new.is_none()) || text_of(g.get("node")) != node {
        return Err(Error::new(
            ErrorKind::Malformed,
            format!(
            "the form names node '{node}' in its content and '{}' in its original — one of them \
             has been edited, and which node this is for cannot be trusted",
            text_of(g.get("node"))
        ),
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
        return Err(Error::new(
            ErrorKind::Malformed,
            format!(
                "[filled_by] ai = \"{ai}\" is not one of {}. Say how an assistant helped — the \
             answer decides what may be applied",
                AI_HELP.join(", ")
            ),
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
                Err(e) => Verdict::Refused(e.into()),
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
pub fn plan_onto(root: &Path, f: &Form, id: &str, like: &str) -> Result<Plan, Error> {
    let tree = crate::load::load_all(root).map_err(|e| e.within("the tree does not load"))?;
    let sh = tree.sheets.get(id).ok_or_else(|| {
        Error::new(
            ErrorKind::Malformed,
            format!("the new node '{id}' is not in the tree"),
        )
    })?;
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
pub fn plan(root: &Path, html: &str) -> Result<Plan, Error> {
    plan_form(root, read(html)?)
}

/// The same, for a form already read.
pub fn plan_form(root: &Path, f: Form) -> Result<Plan, Error> {
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
            Err(e) => bad.push(e.into()),
            Ok((id, _)) if !known.contains(id.as_str()) => bad.push(format!(
                "{id} is not registered — a risk is registered first, on a risk-register row's form"
            )),
            Ok(_) => {}
        }
    }
    (!bad.is_empty()).then(|| bad.join("; "))
}

/// Today, as YYYY-MM-DD, from the system clock (`vleo_units::calendar`).
pub fn today() -> String {
    vleo_units::calendar::Civil::from_unix(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
    )
    .date()
    .to_string()
}

/// A form, planned; `first` when it is a new node's first version.
///
/// EVERY CHANGE TO WHAT A NODE COMPUTES SAYS WHICH BELIEF BROKE. A form whose
/// changes move a decision — an input, the output, the model, the maths, the
/// algorithm, how it is drawn — and does not carry a complete de-risking record
/// has those changes WITHHELD, each named, and only its wording goes in. With a
/// complete record, the changes go in and a `[[version]]` is appended to the
/// sheet, numbered, dated, and `next` until a release stamps it.
fn plan_form_as(root: &Path, f: Form, first: bool) -> Result<Plan, Error> {
    let tree = crate::load::load_all(root).map_err(|e| e.within("the tree does not load"))?;
    plan_form_in(&tree, f, first)
}

/// What a filled form would change, against a tree already loaded — from the
/// folders or from a design file. Writes nothing.
pub fn plan_in(tree: &Tree, html: &str) -> Result<Plan, Error> {
    plan_form_in(tree, read(html)?, false)
}

/// What a form already read would change, against a tree already loaded —
/// from the folders, a design file, or today's design as it is being built
/// from the groups' releases. Writes nothing; the sheet it would write is
/// [`Plan::text`].
pub fn plan_form_against(tree: &Tree, f: Form) -> Result<Plan, Error> {
    plan_form_in(tree, f, false)
}

fn plan_form_in(tree: &Tree, f: Form, first: bool) -> Result<Plan, Error> {
    let root = tree.root.as_path();
    let unlisted = sources_unlisted(tree, &f);
    let mut p = plan_form_on(tree, f, first)?;
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
        form::refuse_agent_attribution(root, &who).map_err(|e| e.within("the method"))?;
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

fn plan_form_on(tree: &Tree, f: Form, first: bool) -> Result<Plan, Error> {
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
fn plan_edits(tree: &Tree, f: Form) -> Result<Plan, Error> {
    let sh = tree.sheets.get(&f.node).ok_or_else(|| {
        Error::new(
            ErrorKind::Malformed,
            format!(
                "there is no node '{}' in this tree. A form never adds a node: a new row is a \
             developer's act, with `xtask new`",
                f.node
            ),
        )
    })?;
    let before = std::fs::read_to_string(sh.dir.join("node.toml"))
        .map_err(|e| Error::io(sh.dir.display(), e))?;
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
                Err(e) => Verdict::Refused(e.into()),
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
                            Verdict::Refused(e.into())
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
                        Verdict::Refused(e.into())
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
                        Verdict::Refused(e.into())
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
                    Err(e) => Verdict::Refused(e.into()),
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
            Err(e) => return Saved::Refused(e.into()),
        };
        if let Err(e) = form::refuse_agent_attribution(root, &who) {
            return Saved::Refused(e.into());
        }
        match form::stamp_relation(after, &who) {
            Ok(t) => t,
            Err(e) => return Saved::Refused(e.into()),
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

/// The form's parts (`web/pages/node-form.html`), read once.
fn parts() -> &'static crate::shell::Parts {
    static P: std::sync::OnceLock<crate::shell::Parts> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        crate::shell::Parts::parse(
            "web/pages/node-form.html",
            include_str!("../../../web/pages/node-form.html"),
        )
        .unwrap_or_else(|e| panic!("{e}"))
    })
}

/// A part as written.
fn t(name: &str) -> &'static str {
    parts().text(name)
}

/// A part with its slots filled.
fn f(name: &str, slots: &[(&str, &str)]) -> String {
    parts().fill(name, slots)
}

const PAGE_CSS: &str = include_str!("../../../web/pages/node-form.css");

/// The page's behaviour: read the two blocks, draw the form, and save a copy
/// with the data block rewritten. No network, no libraries.
const PAGE_JS: &str = include_str!("../../../web/pages/node-form.js");

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
