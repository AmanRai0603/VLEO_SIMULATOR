//! A branch's breakdown, grown by its owner (docs/briefs/F4.md, F4-a; W13;
//! docs/SYSTEM_MODEL.md, sections 2 and 3).
//!
//! A branch is its group file and the node files beside it: the group file
//! holds the headings and every block framed but not yet issued, and each
//! issued node is a node file of its own. A block is in exactly one file.
//! Every edit here writes the group file and nothing else:
//!
//! - **add** a block under a live block of the branch, open, and **fill** an
//!   open block a part at a time: its question, then each port;
//! - **break down** an issued node: one `wire` per output of the node, in the
//!   group file, from the port of the child that answers it. The node's own
//!   file is not touched: its method, its cases and its signature are its
//!   estimate, and its ports and contract do not change (its outside stays).
//!   It is read as broken down by those wires until F5 issues its file again;
//! - **move**, **rename** and **split** a block framed in the group file;
//! - **fold back** a broken-down node, its answering wires taken out, its
//!   children kept;
//! - **remove** a block: archived, never deleted.
//!
//! **Only the branch's owner frames it**, or their deputy, with a key that
//! counts now (`chain::may_frame`), on a group file that opens through the
//! chain as it is, and each edit says why: a `change` row,
//! its `what` the edit and the reason, with the rows it touched before and
//! after. The file is signed afresh by whoever made the edit and must open
//! through the chain as it will be read, or nothing is given back.
//!
//! **A node is broken down only into children that reproduce its own cases**
//! ([`reproduces`]): its children, run in the order their wires give, on each
//! of the node's cases, give what the case expects within its own tolerance,
//! and refuse where it must refuse. The node's cases are the evidence, read
//! as its checks read them (`checks::method_cases`), and taken only from a
//! node file its checks pass and its node engineer signed as it is; the
//! children's answer is what is under test, each child held to its
//! declaration and its method to its ports before it runs. No formula is written here: each child's method runs
//! in the method language, and agreeing with a case is `judge_outcome`'s
//! rule.
//!
//! Not here: issuing node files, wiring and assigning (F5), mounting a group
//! on a node (F6), the programme's branch (F7). A node issued is moved,
//! renamed, split or removed by issuing it again (F5).

use std::collections::{BTreeMap, BTreeSet};

use crate::chain::{self, Registry, FILE};
use crate::checks::{self, Findings};
use crate::error::{Error, ErrorKind};
use crate::keys::SigningKey;
use crate::meta::Kind;
use crate::model::{Assignment, Block, Change, File, Port, Wire, ALLOWED};
use vleo_sheet::method as m;

fn refused(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Malformed, message)
}

/// One edit to a branch. Blocks are named by their uid.
#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    /// A block, open, under `parent`, by its `id`.
    Add {
        parent: String,
        id: String,
        question: String,
    },
    /// One part of an open block framed in the group file.
    Fill { block: String, part: Part },
    /// An issued node, answered by its children: each of its outputs by one
    /// child's output port.
    BreakDown { node: String, answers: Vec<Answer> },
    /// A block framed in the group file, under another parent, at `ord`.
    Move {
        block: String,
        parent: String,
        ord: i64,
    },
    /// A block framed in the group file, given another id.
    Rename { block: String, id: String },
    /// An open block framed in the group file, its `ports` moved to a new
    /// sibling `id`.
    Split {
        block: String,
        id: String,
        ports: Vec<String>,
    },
    /// A broken-down node answered by its estimate again.
    FoldBack { node: String },
    /// A block framed in the group file, archived.
    Remove { block: String },
    /// A block given to `person`, a node engineer its group file registers:
    /// its one assignment, at the contract it has (F5).
    Assign { block: String, person: String },
    /// The input `port` of a block framed in the group file, read from
    /// `from`: `<block>.<port>` in the branch, `<group>.<node>[.<port>]` in
    /// another, or `case` (F5).
    Wire {
        block: String,
        port: String,
        from: String,
    },
    /// A block framed in the group file, decided enough to be written,
    /// issued as a node file of its own to the node engineer it is assigned
    /// to, as a `method` or a `stated` node: its rows leave the group file
    /// for that file ([`issued`]) (F5).
    Issue { block: String, behaviour: String },
    /// An issued node's contract changed by its branch's owner (W4): its
    /// assignment's contract rises, every node that reads it is named, and
    /// its file is issued again at the new contract with `port` in place of
    /// its port of that direction and name, its content kept ([`contracted`]).
    /// Its node engineer's signature no longer counts: it reads behind until
    /// they sign it again (F5).
    Contract {
        node: String,
        port: Option<Box<Port>>,
    },
    /// The group's base moved to `design`, the released design after the one
    /// it is based on, `previous` (none before its first) (W12): what it
    /// reads from other groups that changed under it, and its mount, are
    /// named for the node engineers who read them; no node's content
    /// changes (F5).
    UpdateBase {
        design: Box<File>,
        previous: Option<Box<File>>,
    },
    /// Many blocks set up at once from a sheet, a CSV a row a block
    /// ([`SHEET_COLUMNS`]): each added, its ports filled, its inputs wired
    /// and its node engineer assigned, as those edits are, each saying the
    /// one why. Every row or none (F5).
    Sheet { csv: String },
}

/// The sheet's columns: a block's id and the block it hangs from; its
/// question; its output, by name, type, unit and range; its inputs,
/// `name:unit=from` each, by `;`, every one a number; and the node engineer
/// it is assigned to. Only `id` and `parent` must be there.
pub const SHEET_COLUMNS: [&str; 10] = [
    "id", "parent", "question", "output", "type", "unit", "lower", "upper", "inputs", "assign",
];

/// The edits a sheet makes, each with its row (the header is row 1) and the
/// id it sets up: every block added and its ports filled first, so a row
/// reads the output of another however they are ordered; then every wire;
/// then every assignment.
pub fn sheet_edits(csv: &str) -> Result<Vec<(usize, String, Edit)>, Error> {
    let (head, rows) = crate::intake::parse_csv(csv);
    if let Some(c) = head.iter().find(|c| !SHEET_COLUMNS.contains(&c.trim())) {
        return Err(refused(format!(
            "the sheet has a column {c:?}, which is not one of its columns: {}",
            SHEET_COLUMNS.join(", ")
        )));
    }
    for need in ["id", "parent"] {
        if !head.iter().any(|c| c.trim() == need) {
            return Err(refused(format!("the sheet has no column {need}")));
        }
    }
    if rows.is_empty() {
        return Err(refused("the sheet has no row: nothing is set up"));
    }
    let (mut frame, mut wires, mut assigns) = (Vec::new(), Vec::new(), Vec::new());
    for (i, r) in rows.iter().enumerate() {
        let n = i + 2;
        let get = |k: &str| {
            head.iter()
                .position(|c| c.trim() == k)
                .and_then(|j| r.get(j))
                .map_or(String::new(), |v| v.trim().to_string())
        };
        let id = get("id");
        let port = |direction: &str, name: String, port_type: String, unit: String| Port {
            direction: direction.into(),
            name,
            port_type,
            unit,
            state: "open".into(),
            ..Port::default()
        };
        frame.push((
            n,
            id.clone(),
            Edit::Add {
                parent: get("parent"),
                id: id.clone(),
                question: get("question"),
            },
        ));
        if !get("output").is_empty() {
            let mut p = port("out", get("output"), get("type"), get("unit"));
            p.lower = get("lower");
            p.upper = get("upper");
            frame.push((
                n,
                id.clone(),
                Edit::Fill {
                    block: id.clone(),
                    part: Part::Port(Box::new(p)),
                },
            ));
        }
        for input in get("inputs")
            .split(';')
            .map(str::trim)
            .filter(|x| !x.is_empty())
        {
            let parsed = input.split_once('=').and_then(|(name, from)| {
                let (name, unit) = name.split_once(':')?;
                Some((name.trim(), unit.trim(), from.trim()))
            });
            let Some((name, unit, from)) =
                parsed.filter(|(a, _, c)| !a.is_empty() && !c.is_empty())
            else {
                return Err(refused(format!(
                    "the sheet's row {n} ({id}): its input {input:?} is not name:unit=from"
                )));
            };
            frame.push((
                n,
                id.clone(),
                Edit::Fill {
                    block: id.clone(),
                    part: Part::Port(Box::new(port(
                        "in",
                        name.into(),
                        "number".into(),
                        unit.into(),
                    ))),
                },
            ));
            wires.push((
                n,
                id.clone(),
                Edit::Wire {
                    block: id.clone(),
                    port: name.into(),
                    from: from.into(),
                },
            ));
        }
        if !get("assign").is_empty() {
            assigns.push((
                n,
                id.clone(),
                Edit::Assign {
                    block: id.clone(),
                    person: get("assign"),
                },
            ));
        }
    }
    frame.extend(wires);
    frame.extend(assigns);
    Ok(frame)
}

impl Edit {
    /// What the edit is, in a word, for its `change` row.
    pub fn name(&self) -> &'static str {
        match self {
            Edit::Add { .. } => "add",
            Edit::Fill { .. } => "fill",
            Edit::BreakDown { .. } => "break down",
            Edit::Move { .. } => "move",
            Edit::Rename { .. } => "rename",
            Edit::Split { .. } => "split",
            Edit::FoldBack { .. } => "fold back",
            Edit::Remove { .. } => "remove",
            Edit::Assign { .. } => "assign",
            Edit::Wire { .. } => "wire",
            Edit::Issue { .. } => "issue",
            Edit::Sheet { .. } => "sheet",
            Edit::Contract { .. } => "contract",
            Edit::UpdateBase { .. } => "update base",
        }
    }
}

/// One part of an open block, filled.
#[derive(Clone, Debug, PartialEq)]
pub enum Part {
    /// The question it answers.
    Question(String),
    /// One of its ports, added, or put in place of the port of the same
    /// direction and name.
    Port(Box<Port>),
}

/// An output of a broken-down node, and the child port that answers it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub output: String,
    pub child: String,
    pub port: String,
}

/// Where a block of the branch is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// Framed in the group file, not issued yet.
    Group,
    /// Issued: a node file of its own, the `n`th of the branch's.
    Node(usize),
}

/// A group's branch: its group file and the node files beside it, read as
/// one tree.
pub struct Branch<'a> {
    pub group: &'a File,
    pub nodes: &'a [File],
}

impl<'a> Branch<'a> {
    pub fn new(group: &'a File, nodes: &'a [File]) -> Branch<'a> {
        Branch { group, nodes }
    }

    fn group_id(&self) -> String {
        self.group.meta.get("group_id").cloned().unwrap_or_default()
    }

    /// The file a place names.
    fn file(&self, place: Place) -> &'a File {
        match place {
            Place::Group => self.group,
            Place::Node(i) => &self.nodes[i],
        }
    }

    /// Every block of the branch, with where it is written.
    pub fn blocks(&self) -> Vec<(&'a Block, Place)> {
        let mut out: Vec<(&Block, Place)> = self
            .group
            .blocks
            .iter()
            .map(|b| (b, Place::Group))
            .collect();
        for (i, f) in self.nodes.iter().enumerate() {
            out.extend(f.blocks.iter().map(|b| (b, Place::Node(i))));
        }
        out
    }

    /// The block `uid`, and where it is written.
    pub fn find(&self, uid: &str) -> Option<(&'a Block, Place)> {
        self.blocks().into_iter().find(|(b, _)| b.uid == uid)
    }

    /// The ports of `uid`, from the file that holds it.
    pub fn ports(&self, uid: &str) -> Vec<&'a Port> {
        match self.find(uid) {
            Some((_, place)) => self
                .file(place)
                .ports
                .iter()
                .filter(|p| p.block_uid == uid)
                .collect(),
            None => Vec::new(),
        }
    }

    /// Every wire of the branch, from any of its files.
    fn wires(&self) -> Vec<&'a Wire> {
        let mut out: Vec<&Wire> = self.group.wires.iter().collect();
        for f in self.nodes {
            out.extend(f.wires.iter());
        }
        out
    }

    /// The wire into `uid`'s port `port`, from any file of the branch.
    fn wire_into(&self, uid: &str, port: &str) -> Option<&'a Wire> {
        self.wires()
            .into_iter()
            .find(|w| w.to_block == uid && w.to_port == port)
    }

    /// The live children of `uid`, in their order.
    pub fn children(&self, uid: &str) -> Vec<&'a Block> {
        let mut out: Vec<&Block> = self
            .blocks()
            .into_iter()
            .map(|(b, _)| b)
            .filter(|b| b.parent_uid == uid && b.archived == 0)
            .collect();
        out.sort_by_key(|b| (b.ord, b.id.clone()));
        out
    }

    /// The outputs of `node` its children answer, from the group file's
    /// answering wires: output, child, child's port.
    pub fn answers(&self, node: &str) -> Vec<Answer> {
        let outs: BTreeSet<String> = self
            .ports(node)
            .iter()
            .filter(|p| p.direction == "out")
            .map(|p| p.name.clone())
            .collect();
        self.group
            .wires
            .iter()
            .filter(|w| w.to_block == node && outs.contains(&w.to_port))
            .filter_map(|w| {
                let (child, port) = w.from_ref.split_once('.')?;
                Some(Answer {
                    output: w.to_port.clone(),
                    child: child.to_string(),
                    port: port.to_string(),
                })
            })
            .collect()
    }

    /// Whether `uid` answers a broken-down node.
    fn answers_a_node(&self, uid: &str) -> Option<String> {
        self.group.wires.iter().find_map(|w| {
            let (child, _) = w.from_ref.split_once('.')?;
            (child == uid && !self.answers(&w.to_block).is_empty()).then(|| w.to_block.clone())
        })
    }

    /// What is still open in `uid`: each part a person has yet to decide.
    pub fn open_parts(&self, uid: &str) -> Vec<String> {
        let Some((b, _)) = self.find(uid) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        if b.question.trim().is_empty() {
            out.push("its question".to_string());
        }
        let ports = self.ports(uid);
        if b.behaviour == "open" {
            out.push("its behaviour".to_string());
            if !ports.iter().any(|p| p.direction == "out") {
                out.push("an output".to_string());
            }
        }
        for p in ports {
            if p.port_type.is_empty() {
                out.push(format!("the type of {}", p.name));
            }
            if p.state.is_empty() || p.state == "open" {
                out.push(format!("the state of {}", p.name));
            }
        }
        out
    }
}

/// A uid or id a person can give: lower-case words joined by `_`, starting
/// with a letter, 64 characters at most. A group's id is one too.
pub fn valid_id(id: &str) -> bool {
    let mut c = id.chars();
    matches!(c.next(), Some('a'..='z'))
        && id.len() <= 64
        && id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

fn allowed(table: &str, column: &str, value: &str) -> bool {
    ALLOWED
        .iter()
        .find(|(t, c, _)| *t == table && *c == column)
        .is_none_or(|(_, _, values)| values.contains(&value))
}

/// The rows an edit touched, as text, for its `change` row: each block, port
/// and wire on a line of its own, by its fields.
fn rows_text(blocks: &[&Block], ports: &[&Port], wires: &[&Wire]) -> String {
    let mut o = String::new();
    for b in blocks {
        o.push_str(&format!(
            "block uid={:?} id={:?} parent={:?} behaviour={:?} question={:?} ord={} archived={}\n",
            b.uid, b.id, b.parent_uid, b.behaviour, b.question, b.ord, b.archived
        ));
    }
    for p in ports {
        o.push_str(&format!(
            "port block={:?} {} {:?} type={:?} unit={:?} range={:?}..{:?} state={:?} maturity={:?}\n",
            p.block_uid,
            p.direction,
            p.name,
            p.port_type,
            p.unit,
            p.lower,
            p.upper,
            p.state,
            p.maturity
        ));
    }
    for w in wires {
        o.push_str(&format!(
            "wire {:?}.{:?} from {:?}\n",
            w.to_block, w.to_port, w.from_ref
        ));
    }
    o
}

/// The group file `group` with `edit` made, as `by` with `key` at `at`,
/// saying `why`; signed afresh by `by` and opened again through the chain
/// as it will be read (`open`), or refused with nothing given back. `nodes`
/// are the branch's node files; `elsewhere`, every other file of the
/// database, so a block another group reads is never renamed or removed
/// under it. `mounted_by` is the file that mounts the group, `programme` the
/// programme's file. A breakdown's verdict comes back beside the file.
#[allow(clippy::too_many_arguments)]
pub fn edit(
    group: &File,
    nodes: &[File],
    elsewhere: &[File],
    edit: &Edit,
    why: &str,
    by: &str,
    key: &SigningKey,
    at: &str,
    mounted_by: &Registry,
    programme: &Registry,
) -> Result<(File, Option<Reproduced>), Error> {
    if group.kind()? != Kind::Group {
        return Err(Error::new(
            ErrorKind::WrongKind,
            "a branch is framed in its group file, and this is not one",
        ));
    }
    let id = group.meta.get("group_id").cloned().unwrap_or_default();
    if id == "programme" {
        return Err(refused(
            "the programme's branch is framed in the Programme workspace (F7), not here",
        ));
    }
    // The file as it is, before anything: an edit is signed afresh over every
    // row of it, so one that does not open now is never signed into one that
    // does.
    let registry = chain::open_group_in(group, mounted_by, programme).map_err(|e| {
        refused(format!(
            "{id}'s group file does not open through the chain as it is, so nothing is framed on it: {}",
            e.message()
        ))
    })?;
    chain::may_frame(&id, by, &key.public().to_hex(), at, mounted_by, programme)?;
    if why.trim().is_empty() {
        return Err(refused(format!(
            "say why: every change to {id}'s breakdown says which belief it follows from, and this one says nothing"
        )));
    }
    if checks::is_assistant(by) {
        return Err(refused(format!(
            "{by} is an assistant's name: a breakdown is a person's decision"
        )));
    }
    for f in nodes {
        if f.meta.get("group_id") != Some(&id) {
            return Err(refused(format!(
                "a node file of {} is given as {id}'s",
                f.meta.get("group_id").map_or("no group", String::as_str)
            )));
        }
    }
    // A released design is the system engineer's: each is checked through
    // the chain before the base moves to it.
    if let Edit::UpdateBase { design, previous } = edit {
        for d in std::iter::once(design).chain(previous.iter()) {
            chain::check_design(d, programme)?;
        }
    }
    // A sheet is the edits it makes, each made as it is alone, in order:
    // the first refused refuses the sheet, by its row.
    if let Edit::Sheet { csv } = edit {
        let mut g = group.clone();
        for (row, id, e) in sheet_edits(csv)? {
            g = self::edit(
                &g, nodes, elsewhere, &e, why, by, key, at, mounted_by, programme,
            )
            .map_err(|err| {
                refused(format!(
                    "the sheet's row {row} ({id}) is refused, so nothing is set up: {}",
                    err.message()
                ))
            })?
            .0;
        }
        return Ok((g, None));
    }
    let branch = Branch::new(group, nodes);
    let mut out = group.clone();
    let (scope, before, verdict) = apply(&branch, elsewhere, edit, &mut out, &registry)?;
    let after_branch = Branch::new(&out, nodes);
    let mut after = touched(&after_branch, &scope);
    // A split makes a block: its rows are what the edit made, too.
    if let Edit::Split { id, .. } = edit {
        after.push_str(&touched(&after_branch, id));
    }
    // An assignment is the block's row in the group file: what it was, and is.
    if let Edit::Assign { block, .. } = edit {
        after.push_str(&assigned(&out, block));
    }
    // A contract's assignment as it is now, and who reads the node.
    if let Edit::Contract { node, .. } = edit {
        after.push_str(&assigned(&out, node));
        let readers = readers_of(&branch, elsewhere, node);
        after.push_str(&if readers.is_empty() {
            "read by nothing\n".to_string()
        } else {
            format!("read by {}\n", readers.join(", "))
        });
    }
    // The base as it is now, and what changed under it.
    if let Edit::UpdateBase { design, previous } = edit {
        after.push_str(&format!(
            "based_on={:?} base_version={}\n",
            out.meta.get("based_on").map_or("", String::as_str),
            out.base_version().unwrap_or(0)
        ));
        for line in base_moves(&branch, previous.as_deref(), design) {
            after.push_str(&line);
            after.push('\n');
        }
    }
    // An issue's rows are in the node file it makes, not the group file.
    if let Edit::Issue { block, behaviour } = edit {
        let n = issued(group, block, behaviour)?;
        after.push_str(&format!(
            "issued to nodes/{}.vnode, written by {} at contract {}\n",
            n.blocks[0].id,
            n.meta.get("writer").map_or("", String::as_str),
            n.meta.get("contract_version").map_or("", String::as_str)
        ));
    }
    // What the breakdown was judged on: the verdict, and each child it ran
    // by the digest of its rows, so it can be judged again once one changes.
    if let Some(v) = &verdict {
        after.push_str(&format!("reproduces: {}\n", v.says()));
        for (child, digest) in &v.children {
            after.push_str(&format!("child {child:?} rows {digest}\n"));
        }
    }
    out.changes.push(Change {
        at: at.to_string(),
        who: by.to_string(),
        scope: scope.clone(),
        what: format!("{}: {}", edit.name(), why.trim()),
        before: (!before.is_empty()).then_some(before),
        after: (!after.is_empty()).then_some(after),
    });
    // What was signed was the file as it was: signed afresh by whoever made
    // the edit, and opened as it will be read.
    out.signatures.retain(|s| s.scope != FILE);
    let row = chain::sign(&out, FILE, by, key, at, "ok", "")?;
    out.signatures.push(row);
    out.check_meta()?;
    chain::open_group_in(&out, mounted_by, programme).map_err(|e| {
        refused(format!(
            "the edit would leave {id}'s group file refused when it is next opened, so it is not made: {}",
            e.message()
        ))
    })?;
    Ok((out, verdict))
}

/// The rows of `scope` in `branch` as text: its block, its ports and the
/// wires into it, or answering it.
fn touched(branch: &Branch, scope: &str) -> String {
    let blocks: Vec<&Block> = branch.find(scope).map(|(b, _)| b).into_iter().collect();
    let ports: Vec<&Port> = branch
        .group
        .ports
        .iter()
        .filter(|p| p.block_uid == scope)
        .collect();
    let wires: Vec<&Wire> = branch
        .group
        .wires
        .iter()
        .filter(|w| w.to_block == scope || w.from_ref.starts_with(&format!("{scope}.")))
        .collect();
    rows_text(&blocks, &ports, &wires)
}

/// The node file `block`, framed in the group file `group`, is issued as:
/// its block, as a `behaviour` node at revision 1 and the contract its
/// assignment holds, and every row of it in the group file; its writer the
/// node engineer it is assigned to. Signed by nobody: its node engineer
/// writes it and signs it.
pub fn issued(group: &File, block: &str, behaviour: &str) -> Result<File, Error> {
    let b = group
        .blocks
        .iter()
        .find(|b| b.uid == block)
        .ok_or_else(|| refused(format!("the group file frames no block {block:?}")))?;
    let a = group
        .assignments
        .iter()
        .find(|a| a.block_uid == block)
        .ok_or_else(|| refused(format!("{} is assigned to nobody: assign it first", b.id)))?;
    let id = group.meta.get("group_id").cloned().unwrap_or_default();
    let app = group
        .meta
        .get("written_by_app")
        .cloned()
        .unwrap_or_default();
    let mut n = File::new(Kind::Node, &app);
    for (k, v) in [
        ("group_id", id.as_str()),
        ("block_uid", block),
        ("revision", "1"),
        ("contract_version", &a.contract_version.to_string()),
        ("writer", a.person.as_str()),
    ] {
        n.meta.insert(k.into(), v.to_string());
    }
    let mut nb = b.clone();
    nb.behaviour = behaviour.to_string();
    nb.revision = 1;
    nb.contract_version = a.contract_version;
    n.blocks = vec![nb];
    let mine = |x: &str| x == block;
    n.ports = group
        .ports
        .iter()
        .filter(|r| mine(&r.block_uid))
        .cloned()
        .collect();
    n.wires = group
        .wires
        .iter()
        .filter(|r| mine(&r.to_block))
        .cloned()
        .collect();
    n.closures = group
        .closures
        .iter()
        .filter(|r| mine(&r.block_uid))
        .cloned()
        .collect();
    n.loops = group
        .loops
        .iter()
        .filter(|r| mine(&r.block_uid))
        .cloned()
        .collect();
    n.cases = group
        .cases
        .iter()
        .filter(|r| mine(&r.block_uid))
        .cloned()
        .collect();
    n.texts = group
        .texts
        .iter()
        .filter(|r| mine(&r.scope))
        .cloned()
        .collect();
    n.tables = group
        .tables
        .iter()
        .filter(|r| mine(&r.scope))
        .cloned()
        .collect();
    n.media = group
        .media
        .iter()
        .filter(|r| mine(&r.scope))
        .cloned()
        .collect();
    Ok(n)
}

/// Every input that reads the node `uid` of `branch`, by any of its ports:
/// `<node>'s input <port>`, in the branch or in another group's file.
fn readers_of(branch: &Branch, elsewhere: &[File], uid: &str) -> Vec<String> {
    let Some((b, _)) = branch.find(uid) else {
        return Vec::new();
    };
    let here = format!("{uid}.");
    let there = format!("{}.{}", branch.group_id(), b.id);
    let mut out: Vec<String> = branch
        .wires()
        .into_iter()
        .filter(|w| w.to_block != uid && w.from_ref.starts_with(&here))
        .map(|w| format!("{}'s input {}", w.to_block, w.to_port))
        .collect();
    for f in elsewhere {
        for w in &f.wires {
            if w.from_ref == there || w.from_ref.starts_with(&format!("{there}.")) {
                out.push(format!("{}'s input {}", w.to_block, w.to_port));
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// The node file `node` issued again under the contract `group` now holds
/// for it: its block and its meta at that contract, `port` in place of its
/// port of that direction and name (or added), and everything else as it
/// was, its signatures with it. They were made over the node as it was, so
/// none counts until its node engineer signs it again.
pub fn contracted(node: &File, group: &File, port: Option<&Port>) -> Result<File, Error> {
    let b = node
        .blocks
        .first()
        .ok_or_else(|| refused("a node file holds no block"))?;
    let a = group
        .assignments
        .iter()
        .find(|a| a.block_uid == b.uid)
        .ok_or_else(|| refused(format!("{} is assigned to nobody", b.id)))?;
    let mut n = node.clone();
    n.blocks[0].contract_version = a.contract_version;
    n.meta
        .insert("contract_version".into(), a.contract_version.to_string());
    if let Some(p) = port {
        let mut p = p.clone();
        p.block_uid = b.uid.clone();
        match n
            .ports
            .iter()
            .position(|x| x.direction == p.direction && x.name == p.name)
        {
            Some(i) => {
                p.ord = n.ports[i].ord;
                n.ports[i] = p;
            }
            None => n.ports.push(p),
        }
    }
    Ok(n)
}

/// What changed under a branch between the released design `old` (none
/// before its first) and `new`: each input of the branch that reads another
/// group's port whose type, unit or range differs, or that is gone, said for
/// the node engineer who reads it; and the group's mount, moved.
fn base_moves(branch: &Branch, old: Option<&File>, new: &File) -> Vec<String> {
    let group = branch.group_id();
    let port_of = |d: &File, node: &str, port: &str| -> Option<Port> {
        let b = d.blocks.iter().find(|b| b.id == node)?;
        d.ports
            .iter()
            .find(|p| p.block_uid == b.uid && p.direction == "out" && p.name == port)
            .cloned()
    };
    let mut out = Vec::new();
    let Some(old) = old else {
        return out;
    };
    for w in branch.wires() {
        let parts: Vec<&str> = w.from_ref.split('.').collect();
        let (g, node, port) = match parts.as_slice() {
            [g, node, port] => (*g, *node, *port),
            [g, node] if branch.find(g).is_none() => (*g, *node, *node),
            _ => continue,
        };
        if g == group || g == "case" {
            continue;
        }
        let reads = format!("{}'s input {} reads {}", w.to_block, w.to_port, w.from_ref);
        match (port_of(old, node, port), port_of(new, node, port)) {
            (Some(a), Some(b)) => {
                let mut said = Vec::new();
                for (what, x, y) in [
                    ("type", &a.port_type, &b.port_type),
                    ("unit", &a.unit, &b.unit),
                    ("lower", &a.lower, &b.lower),
                    ("upper", &a.upper, &b.upper),
                ] {
                    if x != y {
                        said.push(format!("{what} {x} → {y}"));
                    }
                }
                if !said.is_empty() {
                    out.push(format!(
                        "{reads}, whose meaning changed ({}): check it",
                        said.join(", ")
                    ));
                }
            }
            (Some(_), None) => out.push(format!(
                "{reads}, which the new design no longer holds: check it"
            )),
            (None, Some(_)) => out.push(format!(
                "{reads}, which the design it was based on did not hold: check it"
            )),
            (None, None) => {}
        }
    }
    let mount = |d: &File| {
        d.mounts
            .iter()
            .find(|m| m.group_id == group)
            .map(|m| m.block_uid.clone())
    };
    if let (Some(a), Some(b)) = (mount(old), mount(new)) {
        if a != b {
            out.push(format!("its mount moved from {a} to {b}"));
        }
    }
    out
}

/// The assignment of `uid` in `f`, as text, for a `change` row.
fn assigned(f: &File, uid: &str) -> String {
    f.assignments
        .iter()
        .filter(|a| a.block_uid == uid)
        .map(|a| {
            format!(
                "assignment block={:?} person={:?} contract={}\n",
                a.block_uid, a.person, a.contract_version
            )
        })
        .collect()
}

/// The edit made in `out`, the group file: the uid it is about, the rows it
/// touched as they were, and a breakdown's verdict.
fn apply(
    branch: &Branch,
    elsewhere: &[File],
    edit: &Edit,
    out: &mut File,
    registry: &Registry,
) -> Result<(String, String, Option<Reproduced>), Error> {
    let group_id = branch.group_id();
    let mounts: BTreeSet<&str> = branch
        .group
        .mounts
        .iter()
        .map(|m| m.block_uid.as_str())
        .collect();
    let live = |uid: &str| -> Result<(&Block, Place), Error> {
        let (b, place) = branch
            .find(uid)
            .ok_or_else(|| refused(format!("{group_id}'s branch has no block {uid:?}")))?;
        if b.archived != 0 {
            return Err(refused(format!("{} is removed: it is archived", b.id)));
        }
        if mounts.contains(uid) {
            return Err(refused(format!(
                "{} carries a mount: a group hangs there, and a mount is changed in F6",
                b.id
            )));
        }
        Ok((b, place))
    };
    // A block framed in the group file, not issued: what move, rename, split
    // and remove change.
    let framed = |uid: &str, what: &str| -> Result<&Block, Error> {
        let (b, place) = live(uid)?;
        if place != Place::Group {
            return Err(refused(format!(
                "{} is issued, a node file of its own: it is {what} by issuing it again (F5), not here",
                b.id
            )));
        }
        Ok(b)
    };
    let taken = |id: &str| {
        branch
            .blocks()
            .iter()
            .any(|(b, _)| b.uid == id || b.id == id)
            // A uid is the design's, not the group's: two files holding one
            // are refused when the design is read whole (`n2::Design::of`).
            || elsewhere
                .iter()
                .any(|f| f.blocks.iter().any(|b| b.uid == id))
    };
    let new_id = |id: &str| -> Result<(), Error> {
        if !valid_id(id) {
            return Err(refused(format!(
                "{id:?} is not an id: lower-case words joined by _, starting with a letter"
            )));
        }
        if taken(id) {
            return Err(refused(format!(
                "{id} is in {group_id}'s branch or another file of the design already, live or removed: an id is never used twice"
            )));
        }
        Ok(())
    };
    // Who reads a block's port `port`, or any of its ports: a wire anywhere
    // in the branch, or in another group by `<group>.<id>.`. A wire into an
    // output is a child answering its node, not a reader.
    let read_by = |b: &Block, port: Option<&str>| -> Option<String> {
        let reads = |from: &str, head: &str| match from.strip_prefix(head) {
            Some(rest) => port.is_none_or(|p| rest == p),
            None => false,
        };
        let here = format!("{}.", b.uid);
        let there = format!("{group_id}.{}.", b.id);
        let answering = |w: &Wire| {
            branch
                .ports(&w.to_block)
                .iter()
                .any(|p| p.direction == "out" && p.name == w.to_port)
        };
        branch
            .wires()
            .into_iter()
            .filter(|w| w.to_block != b.uid)
            .find(|w| reads(&w.from_ref, &here) && !answering(w))
            .map(|w| format!("{}'s input {}", w.to_block, w.to_port))
            .or_else(|| {
                elsewhere.iter().find_map(|f| {
                    f.wires
                        .iter()
                        .find(|w| reads(&w.from_ref, &there))
                        .map(|w| format!("{}'s input {}", w.to_block, w.to_port))
                })
            })
    };
    match edit {
        Edit::Add {
            parent,
            id,
            question,
        } => {
            let (p, _) = live(parent)?;
            new_id(id)?;
            let ord = branch
                .children(parent)
                .iter()
                .map(|b| b.ord)
                .max()
                .map_or(0, |o| o + 1);
            out.blocks.push(Block {
                uid: id.clone(),
                id: id.clone(),
                parent_uid: parent.clone(),
                question: question.trim().to_string(),
                behaviour: "open".into(),
                perspective: p.perspective.clone(),
                ord,
                archived: 0,
                contract_version: 1,
                revision: 0,
            });
            Ok((id.clone(), String::new(), None))
        }
        Edit::Fill { block, part } => {
            let b = framed(block, "filled")?;
            if b.behaviour != "open" {
                return Err(refused(format!(
                    "{} is decided, not open: a part of it changes by a contract change (W4), asked for",
                    b.id
                )));
            }
            let before = touched(branch, block);
            match part {
                Part::Question(q) => {
                    if q.trim().is_empty() {
                        return Err(refused(format!(
                            "{}'s question is filled with words, and none are given",
                            b.id
                        )));
                    }
                    for x in out.blocks.iter_mut().filter(|x| x.uid == *block) {
                        x.question = q.trim().to_string();
                    }
                }
                Part::Port(p) => {
                    let mut p = (**p).clone();
                    p.block_uid = block.clone();
                    port_ok(&p, &b.id)?;
                    // In its place when it replaces one; after the others when new.
                    let at = out.ports.iter().position(|x| {
                        x.block_uid == *block && x.direction == p.direction && x.name == p.name
                    });
                    match at {
                        Some(i) => {
                            p.ord = out.ports[i].ord;
                            out.ports[i] = p;
                        }
                        None => {
                            p.ord = out
                                .ports
                                .iter()
                                .filter(|x| x.block_uid == *block)
                                .map(|x| x.ord + 1)
                                .max()
                                .unwrap_or(0);
                            out.ports.push(p);
                        }
                    }
                }
            }
            Ok((block.clone(), before, None))
        }
        Edit::BreakDown { node, answers } => {
            let (n, place) = live(node)?;
            if place == Place::Group {
                return Err(refused(format!(
                    "{} is not issued yet: an open block is filled, and a node is broken down once it has a file of its own",
                    n.id
                )));
            }
            if !branch.answers(node).is_empty() {
                return Err(refused(format!(
                    "{} is broken down already: fold it back first, and break it down again",
                    n.id
                )));
            }
            let outs: Vec<&Port> = branch
                .ports(node)
                .into_iter()
                .filter(|p| p.direction == "out")
                .collect();
            if outs.is_empty() {
                return Err(refused(format!(
                    "{} has no output for its children to answer",
                    n.id
                )));
            }
            for o in &outs {
                let given = answers.iter().filter(|a| a.output == o.name).count();
                if given != 1 {
                    return Err(refused(format!(
                        "each output of {} is answered by one child: {} is answered by {given}",
                        n.id, o.name
                    )));
                }
            }
            for a in answers {
                if !outs.iter().any(|o| o.name == a.output) {
                    return Err(refused(format!("{} has no output {}", n.id, a.output)));
                }
                let (c, _) = live(&a.child)?;
                if c.parent_uid != *node {
                    return Err(refused(format!(
                        "{} is not a child of {}: a node is answered by its own children",
                        c.id, n.id
                    )));
                }
                let couts: Vec<&Port> = branch
                    .ports(&a.child)
                    .into_iter()
                    .filter(|p| p.direction == "out")
                    .collect();
                if !couts.iter().any(|p| p.name == a.port) {
                    return Err(refused(format!("{} has no output {}", c.id, a.port)));
                }
                // Its first output is its answer; the others are members it
                // publishes beside it, which are not compared yet.
                if couts.first().is_some_and(|p| p.name != a.port) {
                    return Err(refused(format!(
                        "{}.{} is a member {} publishes, not its answer: an output is answered by a child's answer, {}, so far",
                        c.id, a.port, c.id, couts[0].name
                    )));
                }
            }
            let before = touched(branch, node);
            for a in answers {
                out.wires.push(Wire {
                    to_block: node.clone(),
                    to_port: a.output.clone(),
                    from_ref: format!("{}.{}", a.child, a.port),
                });
            }
            let verdict = reproduces(&Branch::new(out, branch.nodes), node, Some(registry));
            if let Some(why) = verdict.refusal(&n.id) {
                return Err(refused(why));
            }
            Ok((node.clone(), before, Some(verdict)))
        }
        Edit::Move { block, parent, ord } => {
            let b = framed(block, "moved")?;
            let _ = live(parent)?;
            if let Some(n) = branch.answers_a_node(block) {
                return Err(refused(format!(
                    "{} answers {n}: fold {n} back before it moves",
                    b.id
                )));
            }
            // Never under itself, at any depth.
            let mut up = parent.clone();
            let mut seen = BTreeSet::new();
            while !up.is_empty() && seen.insert(up.clone()) {
                if up == *block {
                    return Err(refused(format!(
                        "{} cannot hang under itself: {parent} is inside it",
                        b.id
                    )));
                }
                up = branch
                    .find(&up)
                    .map_or(String::new(), |(x, _)| x.parent_uid.clone());
            }
            let before = touched(branch, block);
            for x in out.blocks.iter_mut().filter(|x| x.uid == *block) {
                x.parent_uid = parent.clone();
                x.ord = *ord;
            }
            Ok((block.clone(), before, None))
        }
        Edit::Rename { block, id } => {
            let b = framed(block, "renamed")?;
            new_id(id)?;
            if let Some(r) = read_by(b, None) {
                return Err(refused(format!(
                    "{} is read by {r}: it keeps its id while anything reads it",
                    b.id
                )));
            }
            let before = touched(branch, block);
            for x in out.blocks.iter_mut().filter(|x| x.uid == *block) {
                x.id = id.clone();
            }
            Ok((block.clone(), before, None))
        }
        Edit::Split { block, id, ports } => {
            let b = framed(block, "split")?;
            if b.behaviour != "open" {
                return Err(refused(format!(
                    "{} is decided: an open block is split",
                    b.id
                )));
            }
            if let Some(n) = branch.answers_a_node(block) {
                return Err(refused(format!(
                    "{} answers {n}: fold {n} back before it is split",
                    b.id
                )));
            }
            new_id(id)?;
            let has: Vec<&Port> = branch.ports(block);
            for p in ports {
                let named: Vec<&&Port> = has.iter().filter(|x| &x.name == p).collect();
                if named.is_empty() {
                    return Err(refused(format!("{} has no port {p}", b.id)));
                }
                if named.len() > 1 {
                    return Err(refused(format!(
                        "{} has an input and an output named {p}: which one moves is not said, so neither does",
                        b.id
                    )));
                }
                // An output moves only while nothing reads it.
                if named[0].direction == "out" {
                    if let Some(r) = read_by(b, Some(p)) {
                        return Err(refused(format!(
                            "{}.{p} is read by {r}: it stays where it is read from",
                            b.id
                        )));
                    }
                }
            }
            if ports.is_empty() {
                return Err(refused(format!(
                    "say which of {}'s ports go to {id}: a split moves some, and none are named",
                    b.id
                )));
            }
            let before = touched(branch, block);
            let mut sibling = b.clone();
            sibling.uid = id.clone();
            sibling.id = id.clone();
            // After every block under its parent: an order is never shared.
            sibling.ord = branch
                .children(&b.parent_uid)
                .iter()
                .map(|x| x.ord + 1)
                .max()
                .unwrap_or(0);
            out.blocks.push(sibling);
            for p in out
                .ports
                .iter_mut()
                .filter(|p| p.block_uid == *block && ports.contains(&p.name))
            {
                p.block_uid = id.clone();
            }
            for w in out
                .wires
                .iter_mut()
                .filter(|w| w.to_block == *block && ports.contains(&w.to_port))
            {
                w.to_block = id.clone();
            }
            Ok((block.clone(), before, None))
        }
        Edit::FoldBack { node } => {
            let (n, _) = live(node)?;
            if branch.answers(node).is_empty() {
                return Err(refused(format!(
                    "{} is not broken down: nothing folds back",
                    n.id
                )));
            }
            let before = touched(branch, node);
            let outs: BTreeSet<String> = branch
                .ports(node)
                .iter()
                .filter(|p| p.direction == "out")
                .map(|p| p.name.clone())
                .collect();
            out.wires
                .retain(|w| !(w.to_block == *node && outs.contains(&w.to_port)));
            Ok((node.clone(), before, None))
        }
        Edit::Assign { block, person } => {
            let (b, place) = live(block)?;
            if place == Place::Group && b.behaviour == "children" {
                return Err(refused(format!(
                    "{} is a heading: it holds the blocks under it, and is its owner's own, assigned to nobody",
                    b.id
                )));
            }
            if !registry
                .file
                .people
                .iter()
                .any(|p| p.name == *person && p.role == "node engineer")
            {
                return Err(refused(format!(
                    "{group_id}'s group file registers no node engineer {person}: a block is assigned to a node engineer its group registers"
                )));
            }
            let at = out.assignments.iter().position(|a| a.block_uid == *block);
            let before = assigned(out, block);
            match at {
                Some(i) if out.assignments[i].person == *person => {
                    return Err(refused(format!("{} is assigned to {person} already", b.id)))
                }
                // Assigned again: the work moves to another person, and the
                // contract stays what it was.
                Some(i) => out.assignments[i].person = person.clone(),
                None => out.assignments.push(Assignment {
                    block_uid: block.clone(),
                    person: person.clone(),
                    contract_version: b.contract_version.max(1),
                }),
            }
            Ok((block.clone(), before, None))
        }
        Edit::Wire { block, port, from } => {
            let b = framed(block, "wired")?;
            if !branch
                .ports(block)
                .iter()
                .any(|p| p.direction == "in" && p.name == *port)
            {
                return Err(refused(format!("{} has no input {port}", b.id)));
            }
            let from = from.trim();
            let parts: Vec<&str> = from.split('.').collect();
            if parts.first() == Some(&block.as_str()) && parts.len() == 2 {
                return Err(refused(format!(
                    "{}'s input {port} reads itself: a block reads what another answers",
                    b.id
                )));
            }
            // Another group's node, as that group's file holds it: its
            // answer, or the output named.
            let there = |g: &str, node: &str, out_port: &str| {
                g != group_id
                    && elsewhere.iter().any(|f| {
                        f.meta.get("group_id").map(String::as_str) == Some(g)
                            && f.blocks.iter().any(|x| {
                                x.id == node
                                    && x.archived == 0
                                    && f.ports.iter().any(|q| {
                                        q.block_uid == x.uid
                                            && q.direction == "out"
                                            && q.name == out_port
                                    })
                            })
                    })
            };
            let comes = match parts.as_slice() {
                ["case"] => true,
                [head, p] => match branch.find(head) {
                    Some((x, _)) => {
                        x.archived == 0
                            && branch.ports(head).iter().any(|q| {
                                q.name == *p
                                    && (q.direction == "out"
                                        || (q.direction == "in" && x.uid == b.parent_uid))
                            })
                    }
                    None => there(head, p, p),
                },
                [g, node, p] => there(g, node, p),
                _ => false,
            };
            if !comes {
                return Err(refused(format!(
                    "{}'s input {port} comes from nothing: {from:?} is not an output of a block of the branch, an input of the block it hangs from, another group's node, or case",
                    b.id
                )));
            }
            let before = touched(branch, block);
            let w = Wire {
                to_block: block.clone(),
                to_port: port.clone(),
                from_ref: from.to_string(),
            };
            match out
                .wires
                .iter()
                .position(|x| x.to_block == *block && x.to_port == *port)
            {
                Some(i) => out.wires[i] = w,
                None => out.wires.push(w),
            }
            Ok((block.clone(), before, None))
        }
        Edit::Issue { block, behaviour } => {
            let (b, place) = live(block)?;
            if place != Place::Group {
                return Err(refused(format!(
                    "{} is issued already: it is a node file of its own",
                    b.id
                )));
            }
            if b.behaviour == "children" {
                return Err(refused(format!(
                    "{} is a heading: it holds the blocks under it, and stays in the group file",
                    b.id
                )));
            }
            if !matches!(behaviour.as_str(), "method" | "stated") {
                return Err(refused(format!(
                    "{} is issued as a method or a stated node, and {behaviour:?} is neither",
                    b.id
                )));
            }
            if let Some(c) = branch.children(block).first() {
                return Err(refused(format!(
                    "{} has blocks under it, {} among them: a node is issued with nothing framed under it",
                    b.id, c.id
                )));
            }
            if b.question.trim().is_empty() {
                return Err(refused(format!(
                    "{} has no question: a node is issued to answer one",
                    b.id
                )));
            }
            let ports = branch.ports(block);
            if !ports.iter().any(|p| p.direction == "out") {
                return Err(refused(format!(
                    "{} has no output: a node is issued to answer with one",
                    b.id
                )));
            }
            for p in &ports {
                if p.port_type.is_empty() {
                    return Err(refused(format!("{}'s port {} says no type", b.id, p.name)));
                }
                if p.direction == "in" && branch.wire_into(block, &p.name).is_none() {
                    return Err(refused(format!(
                        "{}: its input {} does not say where it comes from: wire it first",
                        b.id, p.name
                    )));
                }
            }
            if !out.assignments.iter().any(|a| a.block_uid == *block) {
                return Err(refused(format!(
                    "{} is assigned to nobody: assign it to its node engineer first",
                    b.id
                )));
            }
            let before = touched(branch, block);
            let mine = |x: &str| x == block.as_str();
            out.blocks.retain(|r| !mine(&r.uid));
            out.ports.retain(|r| !mine(&r.block_uid));
            out.wires.retain(|r| !mine(&r.to_block));
            out.closures.retain(|r| !mine(&r.block_uid));
            out.loops.retain(|r| !mine(&r.block_uid));
            out.cases.retain(|r| !mine(&r.block_uid));
            out.texts.retain(|r| !mine(&r.scope));
            out.tables.retain(|r| !mine(&r.scope));
            out.media.retain(|r| !mine(&r.scope));
            Ok((block.clone(), before, None))
        }
        Edit::Contract { node, port } => {
            let (b, place) = live(node)?;
            if place == Place::Group {
                return Err(refused(format!(
                    "{} is framed in the group file: its parts are filled while it is open; a contract is an issued node's",
                    b.id
                )));
            }
            if let Some(p) = port {
                let mut p = (**p).clone();
                p.block_uid = node.clone();
                port_ok(&p, &b.id)?;
            }
            let before = assigned(out, node);
            let Some(a) = out.assignments.iter_mut().find(|a| a.block_uid == *node) else {
                return Err(refused(format!(
                    "{} is assigned to nobody: a contract is between the group and the node engineer it is assigned to",
                    b.id
                )));
            };
            a.contract_version += 1;
            Ok((node.clone(), before, None))
        }
        Edit::UpdateBase { design, previous } => {
            let version = design.meta.get("version").cloned().unwrap_or_default();
            let based_on = out.meta.get("based_on").cloned().unwrap_or_default();
            if version == based_on {
                return Err(refused(format!(
                    "{group_id} is already based on the released design {version}"
                )));
            }
            let follows = design.meta.get("previous").cloned().unwrap_or_default();
            if follows != based_on {
                return Err(refused(format!(
                    "the released design {version} follows {}, and {group_id} is based on {}: its base moves to each released design in turn",
                    if follows.is_empty() { "none" } else { &follows },
                    if based_on.is_empty() { "none" } else { &based_on }
                )));
            }
            match previous {
                None if !based_on.is_empty() => {
                    return Err(refused(format!(
                        "the released design {based_on}, which {group_id} is based on, is needed to say what changed under it"
                    )))
                }
                Some(p) if p.meta.get("version") != Some(&based_on) => {
                    return Err(refused(format!(
                        "the design given as the one before is {}, and the design it is based on is {based_on}",
                        p.meta.get("version").map_or("", String::as_str)
                    )))
                }
                _ => {}
            }
            let before = format!(
                "based_on={based_on:?} base_version={}\n",
                out.base_version().unwrap_or(0)
            );
            let count = out.base_version().unwrap_or(0) + 1;
            out.meta.insert("based_on".into(), version);
            out.meta.insert("base_version".into(), count.to_string());
            Ok((group_id.to_string(), before, None))
        }
        Edit::Sheet { .. } => Err(refused(
            "a sheet is the edits it makes, each made by itself",
        )),
        Edit::Remove { block } => {
            let b = framed(block, "removed")?;
            if let Some(c) = branch.children(block).first() {
                return Err(refused(format!(
                    "{} holds {}: its children are moved or removed first",
                    b.id, c.id
                )));
            }
            if let Some(n) = branch.answers_a_node(block) {
                return Err(refused(format!(
                    "{} answers {n}: fold {n} back before it is removed",
                    b.id
                )));
            }
            if let Some(r) = read_by(b, None) {
                return Err(refused(format!(
                    "{} is read by {r}: it stays while anything reads it",
                    b.id
                )));
            }
            if let Some(a) = branch
                .group
                .assignments
                .iter()
                .find(|a| a.block_uid == *block)
            {
                return Err(refused(format!(
                    "{} is assigned to {}: it is unassigned first",
                    b.id, a.person
                )));
            }
            let before = touched(branch, block);
            for x in out.blocks.iter_mut().filter(|x| x.uid == *block) {
                x.archived = 1;
            }
            Ok((block.clone(), before, None))
        }
    }
}

/// A port a person fills, held to what the schema allows and what the method
/// language reads.
fn port_ok(p: &Port, block: &str) -> Result<(), Error> {
    if !valid_id(&p.name) {
        return Err(refused(format!(
            "{:?} is not a port's name: lower-case words joined by _, starting with a letter",
            p.name
        )));
    }
    for (column, value) in [
        ("direction", &p.direction),
        ("port_type", &p.port_type),
        ("state", &p.state),
        ("maturity", &p.maturity),
        ("parameter", &p.parameter),
    ] {
        if !allowed("port", column, value) {
            return Err(refused(format!(
                "{block}'s port {} has {column} {value:?}, which the schema does not allow",
                p.name
            )));
        }
    }
    if p.port_type.is_empty() {
        return Err(refused(format!("{block}'s port {} says no type", p.name)));
    }
    if !p.unit.trim().is_empty() {
        m::parse_unit(p.unit.trim()).map_err(|e| {
            refused(format!(
                "{block}'s port {} is in [{}], which the method language does not read: {}",
                p.name,
                p.unit,
                e.message()
            ))
        })?;
    }
    let num = |v: &str| v.trim().parse::<f64>().ok();
    if let (Some(lo), Some(hi)) = (num(&p.lower), num(&p.upper)) {
        if lo > hi {
            return Err(refused(format!(
                "{block}'s port {} runs from {lo} to {hi}: its lower end is above its upper",
                p.name
            )));
        }
    }
    Ok(())
}

/// Whether a node's children reproduce its own cases.
#[derive(Clone, Debug, PartialEq)]
pub struct Reproduced {
    pub state: State,
    /// Each of the node's cases: what it expects, what the children gave,
    /// what the estimate gives, and whether the children agree.
    pub cases: Vec<CaseRow>,
    /// Each child that ran, by id, and the digest of its file's rows: what
    /// the verdict was reached on.
    pub children: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    /// Every case agrees.
    Agrees,
    /// At least one case does not.
    Differs,
    /// The children could not be run on the cases, and why.
    NotRun(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CaseRow {
    pub label: String,
    /// What the case expects, in SI, or `None` where it must refuse.
    pub expected: Option<f64>,
    /// What the children gave, said: a number in SI, a refusal or a fault.
    pub children: String,
    /// What the node's own method, its estimate, gives, said.
    pub estimate: String,
    pub agrees: bool,
    /// The verdict in words.
    pub says: String,
}

impl Reproduced {
    fn not_run(why: String) -> Reproduced {
        Reproduced {
            state: State::NotRun(why),
            cases: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Why a breakdown into these children is refused, in words, or `None`
    /// when every case agrees.
    pub fn refusal(&self, node: &str) -> Option<String> {
        match &self.state {
            State::Agrees => None,
            State::NotRun(why) => Some(format!(
                "{node}'s children cannot be run on its cases, so it is not broken down: {why}"
            )),
            State::Differs => {
                let bad: Vec<String> = self
                    .cases
                    .iter()
                    .filter(|c| !c.agrees)
                    .map(|c| format!("«{}»: {}", c.label, c.says))
                    .collect();
                Some(format!(
                    "{node}'s children do not reproduce its cases, so it is not broken down. {}",
                    bad.join("; ")
                ))
            }
        }
    }

    /// The verdict in a line.
    pub fn says(&self) -> String {
        match &self.state {
            State::Agrees => format!(
                "its children reproduce all {} of its cases",
                self.cases.len()
            ),
            State::Differs => format!(
                "its children reproduce {} of its {} cases",
                self.cases.iter().filter(|c| c.agrees).count(),
                self.cases.len()
            ),
            State::NotRun(why) => format!("not run: {why}"),
        }
    }
}

/// A child, ready to run: its id, its program, its inputs by name in the
/// order the program takes them with where each comes from, and the
/// dimension of its answer.
struct Child {
    uid: String,
    id: String,
    program: m::Program,
    inputs: Vec<(String, Source)>,
}

/// Where a child's input comes from.
enum Source {
    /// The node's own input of this name: the case sets it.
    Node(String),
    /// Another child's answer.
    Sibling(String),
}

/// Whether `node`'s children, as its answering wires in the group file name
/// them, reproduce its own cases (SYSTEM_MODEL, section 2: "Its cases stay,
/// and test the breakdown"). Each child runs its method, in the method
/// language, in the order its wires need; a child's input is the node's own
/// input when it is wired from it, or from where the node's own input comes,
/// and a sibling's answer when it is wired from that sibling. Each case
/// agrees within its own tolerance, by the rule a method's case agrees by.
/// The answer is compared; a published member is not, yet.
///
/// The node's cases are the evidence, so they are taken only from a node
/// file its own checks pass (`checks::check_node`: no expected value from
/// the code under test, at least three cases and a refusal, each a number)
/// and that its node engineer signed as it is now, through `group`, the
/// group file's registry (`chain::check_node`). Without `group`, or either,
/// nothing is run. Each child is held to its declaration (rule 6) and its
/// method to its own ports' units before it runs (`checks::child_ready`).
pub fn reproduces(branch: &Branch, node: &str, group: Option<&Registry>) -> Reproduced {
    let Some((n, place)) = branch.find(node) else {
        return Reproduced::not_run(format!("the branch has no block {node}"));
    };
    let answers = branch.answers(node);
    let Some(answer) = answers.first() else {
        return Reproduced::not_run(format!("{} is not broken down", n.id));
    };
    if answers.len() > 1 {
        return Reproduced::not_run(format!(
            "{} has {} outputs answered: a node with one answer is checked, so far",
            n.id,
            answers.len()
        ));
    }
    let f = branch.file(place);
    let Some(group) = group else {
        return Reproduced::not_run(format!(
            "the group file does not open through the chain, so whether {} is signed by its node engineer cannot be read",
            n.id
        ));
    };
    if let Err(e) = chain::check_node(f, node, group) {
        return Reproduced::not_run(format!(
            "{}'s cases are its node engineer's, and are not taken until they sign it as it is: {}",
            n.id,
            e.message()
        ));
    }
    let own: Vec<String> = checks::check_node(f)
        .errors()
        .map(|e| format!("{}: {}", e.place, e.what))
        .collect();
    if !own.is_empty() {
        return Reproduced::not_run(format!(
            "{}'s own checks refuse it, so its cases are not evidence: {}",
            n.id,
            own.join("; ")
        ));
    }
    let ports = branch.ports(node);
    let ins: Vec<&Port> = ports
        .iter()
        .copied()
        .filter(|p| p.direction == "in")
        .collect();
    let out_port = ports.iter().copied().find(|p| p.direction == "out");
    let mut said = Findings::default();
    let Some((src, sig, cases)) = checks::method_cases(f, n, &ins, out_port, &mut said) else {
        let why = said
            .0
            .first()
            .map(|x| x.what.clone())
            .unwrap_or_else(|| {
                "it holds no method and results to check against (a node of design/'s shape keeps its cases as test_case rows, which this reads in F3-g0)".into()
            });
        return Reproduced::not_run(format!("{}'s own cases do not read: {why}", n.id));
    };
    if cases.is_empty() {
        return Reproduced::not_run(format!("{} has no case: nothing would be compared", n.id));
    }
    let estimate = m::parse(&src).ok();
    // Where each of the node's inputs comes from, so a child wired from the
    // same place reads it. Two inputs from one place leave it unsaid which
    // one a child reads.
    let mut node_from: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for p in &ins {
        if let Some(w) = branch.wire_into(node, &p.name) {
            node_from
                .entry(w.from_ref.clone())
                .or_default()
                .push(p.name.clone());
        }
    }
    let children: Vec<&Block> = branch.children(node);
    let child_uids: BTreeSet<&str> = children.iter().map(|c| c.uid.as_str()).collect();
    let mut ready: Vec<Child> = Vec::new();
    let mut refused_children: BTreeMap<String, String> = BTreeMap::new();
    for c in &children {
        let cports = branch.ports(&c.uid);
        let Some((_, cplace)) = branch.find(&c.uid) else {
            continue;
        };
        let cf = branch.file(cplace);
        let ready_said: Vec<String> = checks::child_ready(cf, c)
            .errors()
            .map(|e| e.what.clone())
            .collect();
        let src = cf
            .texts
            .iter()
            .find(|t| t.scope == c.uid && t.kind == "pseudocode" && !t.body.trim().is_empty());
        let Some(src) = src else {
            // A child nothing reads may stay unrun; one the answer needs is
            // said below, by name.
            continue;
        };
        if !ready_said.is_empty() {
            // Said once it is known to be needed, below.
            refused_children.insert(c.uid.clone(), ready_said.join("; "));
            continue;
        }
        let program = match m::parse(&src.body) {
            Ok(p) => p,
            Err(d) => {
                return Reproduced::not_run(format!(
                    "{}'s method does not read: line {}: {}",
                    c.id, d.line, d.msg
                ))
            }
        };
        let mut inputs = Vec::new();
        for p in cports.iter().filter(|p| p.direction == "in") {
            let Some(w) = branch.wire_into(&c.uid, &p.name) else {
                return Reproduced::not_run(format!(
                    "{}'s input {} is wired from nothing",
                    c.id, p.name
                ));
            };
            let from = &w.from_ref;
            let source = if let Some(input) = from.strip_prefix(&format!("{node}.")) {
                if !ins.iter().any(|q| q.name == input) {
                    return Reproduced::not_run(format!(
                        "{}'s input {} reads {node}.{input}, which {} has no input of",
                        c.id, p.name, n.id
                    ));
                }
                Source::Node(input.to_string())
            } else if let Some(inputs) = node_from.get(from) {
                if inputs.len() > 1 {
                    return Reproduced::not_run(format!(
                        "{}'s input {} reads {from}, which feeds {}'s inputs {} alike: wire it from {node}.<the input it reads>",
                        c.id,
                        p.name,
                        n.id,
                        inputs.join(" and ")
                    ));
                }
                Source::Node(inputs[0].clone())
            } else if let Some((sib, port)) =
                from.split_once('.').filter(|(s, _)| child_uids.contains(s))
            {
                // A sibling's answer, its first output: a member it publishes
                // is not run through, yet.
                let answer = branch
                    .ports(sib)
                    .into_iter()
                    .find(|q| q.direction == "out")
                    .map(|q| q.name.clone());
                if answer.as_deref() != Some(port) {
                    return Reproduced::not_run(format!(
                        "{}'s input {} reads {from}, which is not {sib}'s answer: a member is not run through, yet",
                        c.id, p.name
                    ));
                }
                Source::Sibling(sib.to_string())
            } else {
                return Reproduced::not_run(format!(
                    "{}'s input {} reads {from}, which {}'s cases do not set",
                    c.id, p.name, n.id
                ));
            };
            // The same quantity on both ends.
            let unit = |u: &str| m::parse_unit(if u.trim().is_empty() { "1" } else { u.trim() });
            let want = match &source {
                Source::Node(input) => ins
                    .iter()
                    .find(|q| &q.name == input)
                    .map(|q| q.unit.clone()),
                Source::Sibling(sib) => branch
                    .ports(sib)
                    .iter()
                    .find(|q| q.direction == "out")
                    .map(|q| q.unit.clone()),
            };
            let same = match (unit(&p.unit), want.as_deref().map(unit)) {
                (Ok((fa, da)), Some(Ok((fb, db)))) => da == db && fa == fb,
                _ => false,
            };
            if !same {
                return Reproduced::not_run(format!(
                    "{}'s input {} is in [{}], and what it reads is in [{}]: one quantity, one unit, or a conversion is a method of its own",
                    c.id,
                    p.name,
                    p.unit,
                    want.unwrap_or_else(|| "nothing it says".into())
                ));
            }
            inputs.push((p.name.clone(), source));
        }
        ready.push(Child {
            uid: c.uid.clone(),
            id: c.id.clone(),
            program,
            inputs,
        });
    }
    // The child that answers the node, and every child it needs, in order.
    let mut order: Vec<usize> = Vec::new();
    let mut needed = vec![answer.child.clone()];
    let mut seen = BTreeSet::new();
    while let Some(uid) = needed.pop() {
        if !seen.insert(uid.clone()) {
            continue;
        }
        let Some(i) = ready.iter().position(|c| c.uid == uid) else {
            let id = branch.find(&uid).map_or(uid.clone(), |(b, _)| b.id.clone());
            if let Some(said) = refused_children.get(&uid) {
                return Reproduced::not_run(format!("{id} is not run: {said}"));
            }
            let why = match branch.find(&uid) {
                Some((b, _)) if b.behaviour == "open" => "it is open: not decided yet".to_string(),
                Some((b, _)) => format!("its behaviour is {}, and it holds no method", b.behaviour),
                None => "it is not in the branch".to_string(),
            };
            return Reproduced::not_run(format!("{id} cannot run: {why}"));
        };
        order.push(i);
        for (_, s) in &ready[i].inputs {
            if let Source::Sibling(sib) = s {
                needed.push(sib.clone());
            }
        }
    }
    // The answer in the node's own unit.
    let unit = |u: &str| m::parse_unit(if u.trim().is_empty() { "1" } else { u.trim() });
    let node_unit = out_port.map(|p| p.unit.clone()).unwrap_or_default();
    let child_unit = branch
        .ports(&answer.child)
        .into_iter()
        .find(|q| q.direction == "out")
        .map(|q| q.unit.clone())
        .unwrap_or_default();
    let same = match (unit(&node_unit), unit(&child_unit)) {
        (Ok((fa, da)), Ok((fb, db))) => da == db && fa == fb,
        _ => false,
    };
    if !same {
        return Reproduced::not_run(format!(
            "{} answers in [{node_unit}], and {} in [{child_unit}]: one quantity, one unit",
            n.id, answer.child
        ));
    }
    // Each child after the children it reads.
    let mut sorted: Vec<usize> = Vec::new();
    let mut placed: BTreeSet<String> = BTreeSet::new();
    for _ in 0..=order.len() {
        for &i in &order {
            let c = &ready[i];
            if placed.contains(&c.uid) {
                continue;
            }
            let ok = c.inputs.iter().all(|(_, s)| match s {
                Source::Sibling(sib) => placed.contains(sib),
                Source::Node(_) => true,
            });
            if ok {
                placed.insert(c.uid.clone());
                sorted.push(i);
            }
        }
    }
    if sorted.len() != order.len() {
        return Reproduced::not_run(format!(
            "{}'s children read each other in a loop, which a breakdown is not checked through yet",
            n.id
        ));
    }
    let mut rows = Vec::new();
    for case in &cases {
        let mut answers_of: BTreeMap<String, f64> = BTreeMap::new();
        let mut got: Result<(m::Outcome, Vec<(String, f64)>), m::Diag> = Ok((
            m::Outcome::Refused {
                line: 0,
                reason: String::new(),
            },
            Vec::new(),
        ));
        let mut by = String::new();
        for &i in &sorted {
            let c = &ready[i];
            let mut values = Vec::new();
            for (name, s) in &c.inputs {
                let v = match s {
                    Source::Node(input) => case
                        .inputs
                        .iter()
                        .find(|(k, _)| k == input)
                        .map(|(_, v)| *v),
                    Source::Sibling(sib) => answers_of.get(sib).copied(),
                };
                // A value the case does not give is never made up: nothing
                // is run on a case that does not say it.
                let Some(v) = v else {
                    return Reproduced::not_run(format!(
                        "the case «{}» gives no value for {}, which {} reads",
                        case.label, name, c.id
                    ));
                };
                values.push((name.clone(), v));
            }
            got = m::run_all(&c.program, &values);
            by = c.id.clone();
            match &got {
                Ok((m::Outcome::Answer(v), _)) => {
                    answers_of.insert(c.uid.clone(), *v);
                }
                // A child that refuses, or faults, is what the children give.
                _ => break,
            }
        }
        let mut plain = case.clone();
        plain.also.clear();
        let verdict = m::judge_outcome(got.clone(), &plain);
        let estimate_says = estimate
            .as_ref()
            .map(|p| m::judge(p, &sig, case).text(case))
            .unwrap_or_else(|| "its method does not read".into());
        rows.push(CaseRow {
            label: case.label.clone(),
            expected: case.expect,
            children: match &got {
                Ok((m::Outcome::Answer(v), _)) => m::show(*v),
                Ok((m::Outcome::Refused { reason, .. }, _)) => format!("refused by {by}: {reason}"),
                Err(d) => format!("{by} faults: {d}"),
            },
            estimate: estimate_says,
            agrees: verdict.agrees(),
            says: if verdict.agrees() {
                if case.refuses() {
                    "the children refuse it, as the case says".into()
                } else {
                    "the children agree".into()
                }
            } else {
                format!(
                    "{} (answered by {by})",
                    verdict
                        .text(&plain)
                        .replace("the method", "the children")
                        .replace("your code", "the case")
                )
            },
        });
    }
    let state = if rows.iter().all(|r| r.agrees) {
        State::Agrees
    } else {
        State::Differs
    };
    let children = sorted
        .iter()
        .map(|&i| {
            let c = &ready[i];
            let f = branch
                .find(&c.uid)
                .map(|(_, place)| branch.file(place))
                .expect("a child that ran is in the branch");
            (c.id.clone(), rows_digest(f))
        })
        .collect();
    Reproduced {
        state,
        cases: rows,
        children,
    }
}

/// The digest of a file's rows, as the page carries them (`rows::encode`).
fn rows_digest(f: &File) -> String {
    use crate::keys::{hex, sha256};
    hex(&sha256(&crate::rows::encode(&f.to_tables())))
}

/// `group_id`'s group file in the copy: its bytes, and the file read from
/// those bytes, so what is shown is what its digest names.
#[cfg(not(target_arch = "wasm32"))]
pub fn group_file_in(copy: &std::path::Path, group_id: &str) -> Result<(Vec<u8>, File), Error> {
    if !valid_id(group_id) {
        return Err(refused(format!("{group_id:?} is not a group's id")));
    }
    let rel = format!("groups/{group_id}/{group_id}.vgroup");
    let bytes = std::fs::read(copy.join(&rel))
        .map_err(|e| Error::new(ErrorKind::Io, format!("{rel}: {e}")))?;
    let f = crate::save::parsed(&bytes)?;
    Ok((bytes, f))
}

/// The node files of `group_id`'s branch in the copy, in order: every
/// `.vnode` of its `nodes` folder, opened as the application opens a file,
/// each a node file of that group, or refused by its name. A file that
/// cannot be read is never passed over: the branch would be shown, and
/// judged, without it. What the branch is shown from and what an edit is
/// made on are read by this one reader.
#[cfg(not(target_arch = "wasm32"))]
pub fn nodes_in(
    copy: &std::path::Path,
    group_id: &str,
    how: &crate::upgrade::Upgrade,
) -> Result<Vec<File>, Error> {
    if !valid_id(group_id) {
        return Err(refused(format!("{group_id:?} is not a group's id")));
    }
    let rel = format!("groups/{group_id}/nodes");
    let dir = copy.join(&rel);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let io = |e: std::io::Error| Error::new(ErrorKind::Io, format!("{rel}: {e}"));
    let mut paths = Vec::new();
    for e in std::fs::read_dir(&dir).map_err(io)? {
        let p = e.map_err(io)?.path();
        if p.extension().is_some_and(|x| x == "vnode") {
            paths.push(p);
        }
    }
    paths.sort();
    let mut out = Vec::new();
    for p in paths {
        let name = format!(
            "{rel}/{}",
            p.file_name()
                .map_or(String::new(), |n| n.to_string_lossy().into_owned())
        );
        let f = crate::sqlite::open(&p, how)
            .map_err(|e| refused(format!("{name} cannot be read: {}", e.message())))?
            .file;
        if f.kind().ok() != Some(Kind::Node)
            || f.meta.get("group_id").map(String::as_str) != Some(group_id)
        {
            return Err(refused(format!(
                "{name} is not a node file of {group_id}: a branch's nodes folder holds its node files only"
            )));
        }
        out.push(f);
    }
    Ok(out)
}

/// What an edit in the copy did, or why it wrote nothing.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub enum Framed {
    /// Written: the group file's digest now, and a breakdown's verdict.
    Written {
        digest: String,
        reproduced: Option<Reproduced>,
    },
    /// The group file changed in the copy since it was opened: nothing is
    /// written. Its digest now.
    SecondWriter { digest: String },
}

/// The edit made to `group_id`'s group file inside the copy, when its bytes
/// are still the ones whose SHA-256 is `opened`, under the copy's lock from
/// the first read to the rename, as a save is (`save::save`). The branch's
/// node files and every other file of the copy are read under the same lock.
/// `chain` gives the file that mounts the group and the programme's file, as
/// the copy opens them.
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
pub fn edit_in_place(
    copy: &std::path::Path,
    group_id: &str,
    opened: &str,
    e: &Edit,
    why: &str,
    by: &str,
    key: &SigningKey,
    at: &str,
    app: &str,
    mounted_by: &Registry,
    programme: &Registry,
) -> Result<Framed, Error> {
    if !valid_id(group_id) {
        return Err(refused(format!("{group_id:?} is not a group's id")));
    }
    crate::save::opened_ok(opened)?;
    let _held = crate::save::lock(copy)?;
    let at_file = copy
        .join("groups")
        .join(group_id)
        .join(format!("{group_id}.vgroup"));
    let bytes = std::fs::read(&at_file)
        .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", at_file.display())))?;
    if crate::copied::digest(&bytes) != opened {
        return Ok(Framed::SecondWriter {
            digest: crate::copied::digest(&bytes),
        });
    }
    // What is edited is read from the bytes compared, not from the path
    // again: what the person opened, and nothing that came between.
    let group = crate::save::parsed(&bytes)?;
    let how = crate::upgrade::Upgrade { app, at };
    // The branch's node files, as the branch is shown (`nodes_in`).
    let nodes = nodes_in(copy, group_id, &how)?;
    // Every other file of the copy, each read or the edit refused by its
    // name: a file skipped is a reader missed, or an id used twice.
    let io = |p: &std::path::Path, e: std::io::Error| {
        Error::new(ErrorKind::Io, format!("{}: {e}", p.display()))
    };
    let listed = |dir: &std::path::Path, ext: &str| -> Result<Vec<std::path::PathBuf>, Error> {
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for e in std::fs::read_dir(dir).map_err(|e| io(dir, e))? {
            let p = e.map_err(|e| io(dir, e))?.path();
            if p.extension().is_some_and(|x| x == ext) {
                out.push(p);
            }
        }
        out.sort();
        Ok(out)
    };
    let mut elsewhere = Vec::new();
    let groups = copy.join("groups");
    let mut dirs: Vec<std::path::PathBuf> = Vec::new();
    for g in std::fs::read_dir(&groups).map_err(|e| io(&groups, e))? {
        let g = g.map_err(|e| io(&groups, e))?.path();
        if g.is_dir() {
            dirs.push(g);
        }
    }
    dirs.sort();
    for g in dirs {
        let name = g
            .file_name()
            .map_or(String::new(), |n| n.to_string_lossy().into_owned());
        let mut files = Vec::new();
        if name != group_id {
            files.extend(listed(&g.join("nodes"), "vnode")?);
            let own = g.join(format!("{name}.vgroup"));
            if own.exists() {
                files.push(own);
            }
        }
        // A release another group's work reads from is a reader too.
        files.extend(listed(&g.join("releases"), "vleo")?);
        for p in files {
            let f = crate::sqlite::open(&p, &how)
                .map_err(|e| {
                    refused(format!(
                        "{} cannot be read, so whether the edit leaves anything without what it reads cannot be said: {}",
                        p.strip_prefix(copy).unwrap_or(&p).display(),
                        e.message()
                    ))
                })?
                .file;
            elsewhere.push(f);
        }
    }
    let (mut out, reproduced) = edit(
        &group, &nodes, &elsewhere, e, why, by, key, at, mounted_by, programme,
    )?;
    out.meta.insert("written_by_app".into(), app.to_string());
    // An issue makes the node's own file beside the group file, and never
    // writes over one there.
    let node = match e {
        Edit::Issue { block, behaviour } => {
            let mut n = issued(&group, block, behaviour)?;
            n.meta.insert("written_by_app".into(), app.to_string());
            let at = copy
                .join("groups")
                .join(group_id)
                .join("nodes")
                .join(format!("{}.vnode", n.blocks[0].id));
            if at.exists() {
                return Err(refused(format!(
                    "{} is there already: a node file is never written over",
                    at.strip_prefix(copy).unwrap_or(&at).display()
                )));
            }
            Some((n, at))
        }
        Edit::Contract { node: uid, port } => {
            let at = nodes
                .iter()
                .find(|n| n.blocks.first().is_some_and(|b| b.uid == *uid))
                .map(|n| {
                    copy.join("groups")
                        .join(group_id)
                        .join("nodes")
                        .join(format!("{}.vnode", n.blocks[0].id))
                })
                .ok_or_else(|| refused(format!("{uid} has no node file in the branch")))?;
            let theirs = crate::sqlite::read(&at)?;
            if theirs.blocks.first().map(|b| b.uid.as_str()) != Some(uid.as_str()) {
                return Err(refused(format!(
                    "{} does not hold {uid}: a node's file is named for it",
                    at.strip_prefix(copy).unwrap_or(&at).display()
                )));
            }
            let mut n = contracted(&theirs, &out, port.as_deref())?;
            n.meta.insert("written_by_app".into(), app.to_string());
            Some((n, at))
        }
        _ => None,
    };
    // The one comparison, as late as can be before the write.
    let again = std::fs::read(&at_file)
        .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", at_file.display())))?;
    if crate::copied::digest(&again) != opened {
        return Ok(Framed::SecondWriter {
            digest: crate::copied::digest(&again),
        });
    }
    // The node's file first: a group file without the block's rows is never
    // left with no file holding them.
    // A contract issues the node's file again: what it was is kept, to put
    // back if the group file cannot be written.
    let was = match &node {
        Some((_, at)) if at.exists() => Some(
            std::fs::read(at)
                .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", at.display())))?,
        ),
        _ => None,
    };
    if let Some((n, at)) = &node {
        if let Some(dir) = at.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", dir.display())))?;
        }
        crate::sqlite::written(n, at)?;
    }
    let written = match crate::sqlite::written(&out, &at_file) {
        Ok(w) => w,
        Err(err) => {
            if let Some((_, at)) = &node {
                match &was {
                    Some(bytes) => {
                        let _ = std::fs::write(at, bytes);
                    }
                    None => {
                        let _ = std::fs::remove_file(at);
                    }
                }
            }
            return Err(err);
        }
    };
    Ok(Framed::Written {
        digest: crate::copied::digest(&written),
        reproduced,
    })
}
