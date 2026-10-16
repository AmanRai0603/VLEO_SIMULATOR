//! What a change to a node would do: the plan rules a node's change is held
//! to before anything is written.
//!
//! A change arrives as a [`Form`]: the node as it was when the change began,
//! what the person made of it, who they are, whether an assistant helped and
//! how, and why the node is changing. Today's design builds one for each node
//! of a group's release from before 1.0 (`vleo_files::intake`), and plans it
//! against the design ([`plan_form_against`]). The node form, the HTML file
//! that once carried a change by hand, is retired: its commands went in phase
//! E, and the page itself with F3 (docs/briefs/F3.md, F3-f).
//!
//! THREE VERSIONS, NOT TWO. A form carries the node as it was when the change
//! began, as well as what the person made of it. The plan compares both with
//! the node as it is now, so an edit made in the meantime is never
//! overwritten: a field the change moved that the design also moved is a
//! CONFLICT, named, and left for a person.
//!
//! AN ASSISTANT MAY NEVER SUPPLY MATHEMATICS, at any face. A form says whether
//! an assistant helped, and how. Where it helped with the relation, every
//! change to the relation, its steps and its derivation is refused and
//! reported.

use crate::{Error, ErrorKind};
use std::collections::BTreeMap;

use crate::form::{self, Blocks, ARRAYS, FIELDS};
use crate::load::Tree;
use crate::model::{Sheet, View};

/// How an assistant was involved, as the change declares it: not at all, with
/// the wording, or with the relation; or `transcribed`, a method an assistant
/// copied from a person's source and a person read against it (AGENTS.md,
/// rule 6). Intake gives `transcribed` only when the declaration names both
/// (`vleo_files::intake::declared_help`); the plan rules take the caller's word
/// for it. Saying nothing is not one of them.
pub const AI_HELP: &[&str] = &["none", "wording", "relation", "transcribed"];

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

/// The method checker the pages carry: `vleo_sheet::method::report_toml`
/// compiled to WebAssembly by `xtask method-wasm`, beside the tree's own files.
pub const CHECKER: &str = "web/method.wasm.gz";

/// The checker, read from the tree when a form is made — never built into a
/// program. A program carrying compressed WebAssembly beside the script that
/// unpacks and runs it is the shape browsers and antivirus block, and the
/// first 0.3.0 Windows kit was blocked by Chrome for exactly that. Missing, a
/// page that carries it says so on its check rather than checking by nothing.
/// The lesson form and the group pages carry it.
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
// the content, as a form carries it

/// One node's content in the shape the form carries it: every field as text,
/// every repeated block as rows of text, and how the answer is drawn.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Content {
    pub fields: BTreeMap<String, String>,
    pub arrays: BTreeMap<String, Vec<BTreeMap<String, String>>>,
    /// `(kind, over, points)`, when the node is drawn in a way this form writes.
    pub view: Option<(String, String, String)>,
}

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
    /// What the record is missing.
    pub fn missing(&self) -> Vec<&'static str> {
        [
            "believed",
            "tested",
            "learned",
            "changed",
            "rests_on",
            "breaks_if",
        ]
        .iter()
        .copied()
        .filter(|k| self.get(k).trim().is_empty())
        .collect()
    }
}

/// A filled form, read back.
#[derive(Clone, Debug, Default)]
pub struct Form {
    /// The node it is for. A change never adds a node.
    pub node: String,
    /// The file hash of `node.toml` the form was made from.
    pub base: String,
    pub name: String,
    pub team: String,
    pub date: String,
    /// How an assistant helped: one of [`AI_HELP`].
    pub ai: String,
    pub notes: String,
    pub original: Content,
    pub filled: Content,
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
// what a form would change

/// What happens to one change the form asks for.
#[derive(Clone, Debug, PartialEq)]
pub enum Verdict {
    /// Applied: it goes into the sheet the plan writes ([`Plan::text`]).
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
    /// What the change leaves to be settled that does not stop it: a source it
    /// cites that `sources/` does not list yet.
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

/// Two cells of a repeated block as the sheet holds them: `1e-12` and
/// `0.000000000001` are the same tolerance, and an inline table is the same
/// whatever order its keys are written in. The text a sheet is read as is
/// laid out by whoever wrote it; a block is changed by its values, never by
/// how they were written.
fn same_cell(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let value = |v: &str| format!("v = {v}").parse::<toml::Table>().ok();
    match (value(a), value(b)) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

/// Two lists of blocks the same, cell by cell, as [`same_cell`] reads one.
fn same_rows(a: &[BTreeMap<String, String>], b: &[BTreeMap<String, String>]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|(x, y)| {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| same_cell(v, w)))
        })
}

fn short(v: &str) -> String {
    let one = v.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() > 90 {
        format!("{}…", one.chars().take(89).collect::<String>())
    } else {
        one
    }
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

/// What a form already read would change, against a tree already loaded —
/// from the folders, the design's files, or today's design as it is being
/// built from the groups' releases. Writes nothing; the sheet it would write is
/// [`Plan::text`].
pub fn plan_form_against(tree: &Tree, f: Form) -> Result<Plan, Error> {
    plan_form_in(tree, f)
}

fn plan_form_in(tree: &Tree, f: Form) -> Result<Plan, Error> {
    // How an assistant helped decides what may be applied, so an answer that
    // is not one of these, or none, is refused before anything is planned.
    // Silence is not "none".
    if !AI_HELP.contains(&f.ai.as_str()) {
        return Err(Error::new(
            ErrorKind::Malformed,
            format!(
                "ai = \"{}\" is not one of {}. Say how an assistant helped — the answer decides what may be applied",
                f.ai,
                AI_HELP.join(", ")
            ),
        ));
    }
    let root = tree.root.as_path();
    let unlisted = sources_unlisted(tree, &f);
    let mut p = plan_form_on(tree, f)?;
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
        let mut text = form::set(&text, "method_by", &format!("{who}{when}"))?;
        // A method of the form's own is no longer a transcription: what it
        // was copied from, and whoever read the copy, no longer describe it.
        for key in ["method_transcribed_from", "method_checked_by"] {
            if form::carries(&text, key) {
                text = form::set(&text, key, "")?;
            }
        }
        p.text = Some(text);
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
        if Some(s.as_str()) != was {
            cited.push(("the relation".into(), s.trim().to_string()));
        }
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

fn plan_form_on(tree: &Tree, f: Form) -> Result<Plan, Error> {
    let p = plan_edits(tree, f.clone())?;
    let mut about: Vec<&'static str> = p
        .items
        .iter()
        .filter(|i| i.verdict == Verdict::Apply)
        .filter_map(|i| item_about(&i.what))
        .collect();
    about.sort_by_key(|a| crate::derisk::ABOUT.iter().position(|x| x == a));
    about.dedup();
    if about.is_empty() {
        return Ok(p);
    }
    let missing = f.derisk.missing();
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
            changed: f.derisk.changed.trim().to_string(),
            risks: f.derisk.moves(),
            rests_on: f.derisk.rests_on.trim().to_string(),
            breaks_if: f.derisk.breaks_if.trim().to_string(),
            relation: pick("expression", &sh.expression),
            source: pick("source", &sh.source),
        };
        let base = p.text.clone().unwrap_or_else(|| sh.text.clone());
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
    let before = sh.text.clone();
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
        if same_rows(o, n) {
            continue;
        }
        let c = now.arrays.get(a.name).unwrap_or(&empty);
        let label = |i: usize| format!("{} {}", a.name, i + 1);
        if same_rows(c, n) {
            p.items.push(Item {
                what: a.name.to_string(),
                from: format!("{} block(s)", c.len()),
                to: format!("{} block(s)", n.len()),
                verdict: Verdict::Already,
            });
            continue;
        }
        if !same_rows(c, o) {
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
                if same_cell(&ov, &nv) {
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
