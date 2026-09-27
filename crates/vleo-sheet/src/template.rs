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
//! AN AGENT MAY NEVER SUPPLY MATHEMATICS, at any face. The form asks whether an
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
const RELATION_FIELDS: &[&str] = &["expression", "source", "theory_why", "theory_reading"];
const RELATION_ARRAYS: &[&str] = &["algorithm", "theory"];

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
    for a in ARRAYS {
        let rows = form::array_rows(sh, a)
            .into_iter()
            .map(|r| {
                r.into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect::<BTreeMap<_, _>>()
            })
            .collect();
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
    for (i, a) in ARRAYS.iter().enumerate() {
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
            if i + 1 == ARRAYS.len() { "" } else { "," }
        ));
    }
    o.push_str("  ],\n");
    o.push_str(&format!(
        "  \"choices\": {{\"type\": {}, \"unit\": {}}},\n",
        js_list(vleo_units::QUANTITIES),
        js_list(&crate::unit_names())
    ));
    o.push_str(&format!(
        "  \"view_kinds\": {},\n  \"provenances\": {},\n  \"ai_help\": {},\n",
        js_list(form::VIEW_KINDS),
        js_list(PROVENANCES),
        js_list(AI_HELP)
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
    )
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
    )
}

fn page(title: &str, head: &str, schema: &str, original: &str, data: &str) -> String {
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
         how an assistant helped. On a new node's form, [new] says where it goes: its id, the \
         group it hangs under, and its kind. -->\n<script type=\"application/toml\" id=\"{DATA_ID}\">\n{}</script>\n",
        data
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
    Ok(Form {
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
    let mut p = plan_form(root, g)?;
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
    let tree = crate::load::load_all(root).map_err(|e| format!("the tree does not load: {e}"))?;
    if let Some(n) = f.new.clone() {
        return Ok(plan_new(&tree, f, n));
    }
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
                "the form says an assistant helped with the relation, and an agent may never \
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
        p.interfaces = interfaces(&tree, &n_in);
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
                if RELATION_ARRAYS.contains(&a.name) && verdict == Verdict::Apply {
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
            if RELATION_ARRAYS.contains(&a.name) && verdict == Verdict::Apply {
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
            if RELATION_ARRAYS.contains(&a.name) && verdict == Verdict::Apply {
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
    // relation by applying it — and refuses if that identity is an agent's.
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
pub fn fixture_request(f: &Form) -> String {
    let mut o = String::new();
    for k in &f.known {
        o.push_str("[[fixture]]\n");
        o.push_str(&format!("label = {}\n", tq(&k.label)));
        o.push_str(&format!("expect = {}\n", k.expected.trim()));
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
        o.push_str(&format!(
            "# inputs, as the form gave them: {}\n\n",
            k.inputs
        ));
    }
    o
}

// ---------------------------------------------------------------------------
// the page

const INTRO_HTML: &str = r#"<section class="nf-intro">
<p><b>This is a form for one node of the design, and it is the whole of it</b> — it needs no
connection and nothing installed. Change what should change, say who you are, and save a filled
copy. Send the saved file to the developer team: they check it, apply it to the node, and the next
release of the tool carries your change, where you can run it with your own inputs.</p>
<ol>
<li>Fill in what you know. Each question says <i>why</i> it is asked; a question you cannot
answer, leave as it is.</li>
<li>Say who filled it, and whether an assistant helped. If one helped with the <b>relation</b>
itself — the equation, its steps or its derivation — say so: those are then derived by a developer
rather than taken from the form.</li>
<li>Known-good values — an answer you know at given inputs, and where it comes from — go under
<i>known values</i>. They are recorded by a person, with their source, and never guessed.</li>
<li>Press <b>save a filled copy</b> and send that file. You can reopen it and carry on.</li>
</ol>
<p class="nf-muted">To fill it with an assistant, give it this file and ask it to edit only the
block marked <code>vleo-node-form</code> near the end: plain TOML, one line per answer.</p>
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
.nf-tag { font: 11px var(--mono); padding: 0 6px; border-radius: 3px; margin-left: 6px; vertical-align: 1px;
  border: 1px solid var(--rule); color: var(--ink3); font-weight: 400; }
.nf-tag.req { color: var(--warn); border-color: var(--warn); }
.nf-tag.rel { color: var(--accent); border-color: var(--accent); }
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
    if (S(DATA.notes.text).trim()) out.push('a note for the developers');
    return out;
  }
  function paintCount() {
    const c = changes();
    $('#nf-count').textContent = c.length ? c.length + ' change(s): ' + c.join(', ') : 'nothing changed yet';
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
    } else {
      el = document.createElement('input'); el.type = 'text'; el.value = value;
      if (shape === 'number' || shape === 'count') el.inputMode = 'decimal';
      if (shape === 'row') el.setAttribute('list', 'nf-rows');
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
    paint();
    return box;
  }

  function blocks(a) {
    const wrap = document.createElement('section');
    const rows = DATA[a.name] = DATA[a.name] || [];
    const draw = () => {
      wrap.innerHTML = '<h2>' + esc(a.label.charAt(0).toUpperCase() + a.label.slice(1)) +
        (a.relation ? ' <span class="nf-tag rel">the relation</span>' : '') + '</h2>' +
        '<p class="nf-why">' + esc(a.why) + '</p>';
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
          col.innerHTML = '<span>' + esc(c.ask || c.key) + (c.required ? ' <span class="nf-tag req">needed</span>' : '') + '</span>';
          col.appendChild(control(c.shape, [], S(r[c.key]), v => {
            r[c.key] = v;
            if (a.name === 'input' && c.key === 'var' && !S(r.type) && ROWS.get(v)) {
              r.type = ROWS.get(v)[2]; draw(); return;
            }
            paintHint();
          }, a.name + ' ' + (i + 1) + ' ' + c.key));
          b.appendChild(col);
        }
        if (a.name === 'input') { b.appendChild(hint); paintHint(); }
        wrap.appendChild(b);
      });
      const add = document.createElement('button'); add.type = 'button';
      add.textContent = 'add ' + (a.end_only ? 'one at the end' : 'one');
      add.onclick = () => {
        const r = {}; for (const c of a.columns) r[c.key] = c.managed ? String(rows.length + 1) : '';
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
        const cols = [['label', 'what case this is'], ['inputs', 'the inputs it holds at, e.g. orbit_altitude = 250 km'],
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

  const groups = [];
  for (const f of SCHEMA.fields) if (groups.indexOf(f.group) < 0) groups.push(f.group);
  for (const grp of groups) {
    const sec = document.createElement('section');
    sec.innerHTML = '<h2>' + esc(grp.charAt(0).toUpperCase() + grp.slice(1)) + '</h2>';
    for (const f of SCHEMA.fields.filter(x => x.group === grp)) sec.appendChild(question(f));
    main.appendChild(sec);
  }
  for (const a of SCHEMA.arrays) main.appendChild(blocks(a));

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
