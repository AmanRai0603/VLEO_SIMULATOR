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
//! counts now (`chain::may_frame`), and each edit says why: a `change` row,
//! its `what` the edit and the reason, with the rows it touched before and
//! after. The file is signed afresh by whoever made the edit and must open
//! through the chain as it will be read, or nothing is given back.
//!
//! **A node is broken down only into children that reproduce its own cases**
//! ([`reproduces`]): its children, run in the order their wires give, on each
//! of the node's cases, give what the case expects within its own tolerance,
//! and refuse where it must refuse. The node's cases are the evidence, read
//! as its checks read them (`checks::method_cases`); the children's answer is
//! what is under test. No formula is written here: each child's method runs
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
use crate::model::{Block, Change, File, Port, Wire, ALLOWED};
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
/// with a letter, 64 characters at most.
fn valid_id(id: &str) -> bool {
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
    let branch = Branch::new(group, nodes);
    let mut out = group.clone();
    let (scope, before, verdict) = apply(&branch, elsewhere, edit, &mut out)?;
    let after_branch = Branch::new(&out, nodes);
    let after = touched(&after_branch, &scope);
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

/// The edit made in `out`, the group file: the uid it is about, the rows it
/// touched as they were, and a breakdown's verdict.
fn apply(
    branch: &Branch,
    elsewhere: &[File],
    edit: &Edit,
    out: &mut File,
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
            || elsewhere
                .iter()
                .filter(|f| f.meta.get("group_id") == Some(&group_id))
                .any(|f| f.blocks.iter().any(|b| b.uid == id || b.id == id))
    };
    let new_id = |id: &str| -> Result<(), Error> {
        if !valid_id(id) {
            return Err(refused(format!(
                "{id:?} is not an id: lower-case words joined by _, starting with a letter"
            )));
        }
        if taken(id) {
            return Err(refused(format!(
                "{id} is in {group_id}'s branch already, live or removed: an id is never used twice"
            )));
        }
        Ok(())
    };
    // Who reads a block's ports: a wire anywhere in the branch, or in
    // another group by `<group>.<id>.`.
    let read_by = |b: &Block| -> Option<String> {
        let here = format!("{}.", b.uid);
        let there = format!("{group_id}.{}.", b.id);
        branch
            .wires()
            .into_iter()
            .filter(|w| w.to_block != b.uid)
            .find(|w| w.from_ref.starts_with(&here) && branch.answers(&w.to_block).is_empty())
            .map(|w| format!("{}'s input {}", w.to_block, w.to_port))
            .or_else(|| {
                elsewhere.iter().find_map(|f| {
                    f.wires
                        .iter()
                        .find(|w| w.from_ref.starts_with(&there))
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
                    out.ports.retain(|x| {
                        !(x.block_uid == *block && x.direction == p.direction && x.name == p.name)
                    });
                    out.ports.push(p);
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
                if !branch
                    .ports(&a.child)
                    .iter()
                    .any(|p| p.direction == "out" && p.name == a.port)
                {
                    return Err(refused(format!("{} has no output {}", c.id, a.port)));
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
            let verdict = reproduces(&Branch::new(out, branch.nodes), node);
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
            if let Some(r) = read_by(b) {
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
                if !has.iter().any(|x| &x.name == p) {
                    return Err(refused(format!("{} has no port {p}", b.id)));
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
            sibling.ord = b.ord + 1;
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
            if let Some(r) = read_by(b) {
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
pub fn reproduces(branch: &Branch, node: &str) -> Reproduced {
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
    let estimate = m::parse(&src).ok();
    // Where each of the node's inputs comes from, so a child wired from the
    // same place reads it.
    let node_from: BTreeMap<String, String> = ins
        .iter()
        .filter_map(|p| {
            branch
                .wire_into(node, &p.name)
                .map(|w| (w.from_ref.clone(), p.name.clone()))
        })
        .collect();
    let children: Vec<&Block> = branch.children(node);
    let child_uids: BTreeSet<&str> = children.iter().map(|c| c.uid.as_str()).collect();
    let mut ready: Vec<Child> = Vec::new();
    for c in &children {
        let cports = branch.ports(&c.uid);
        let Some((_, cplace)) = branch.find(&c.uid) else {
            continue;
        };
        let cf = branch.file(cplace);
        let src = cf
            .texts
            .iter()
            .find(|t| t.scope == c.uid && t.kind == "pseudocode" && !t.body.trim().is_empty());
        let Some(src) = src else {
            // A child nothing reads may stay unrun; one the answer needs is
            // said below, by name.
            continue;
        };
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
            } else if let Some(input) = node_from.get(from) {
                Source::Node(input.clone())
            } else if let Some((sib, _)) =
                from.split_once('.').filter(|(s, _)| child_uids.contains(s))
            {
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
            if let (Ok((fa, da)), Some(Ok((fb, db)))) = (unit(&p.unit), want.as_deref().map(unit)) {
                if da != db || fa != fb {
                    return Reproduced::not_run(format!(
                        "{}'s input {} is in [{}], and what it reads is in [{}]: one quantity, one unit, or a conversion is a method of its own",
                        c.id,
                        p.name,
                        p.unit,
                        want.unwrap_or_default()
                    ));
                }
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
                values.push((name.clone(), v.unwrap_or(f64::NAN)));
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
    Reproduced { state, cases: rows }
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
    let dir = copy.join("groups").join(group_id);
    let at_file = dir.join(format!("{group_id}.vgroup"));
    let bytes = std::fs::read(&at_file)
        .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", at_file.display())))?;
    if crate::copied::digest(&bytes) != opened {
        return Ok(Framed::SecondWriter {
            digest: crate::copied::digest(&bytes),
        });
    }
    let group = crate::sqlite::read(&at_file)?;
    let mut nodes = Vec::new();
    let mut elsewhere = Vec::new();
    for g in std::fs::read_dir(copy.join("groups"))
        .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", copy.display())))?
        .flatten()
    {
        let mine = g.file_name().to_string_lossy() == group_id;
        let files = std::fs::read_dir(g.path().join("nodes"))
            .into_iter()
            .flatten()
            .flatten()
            .map(|n| n.path())
            .filter(|p| p.extension().is_some_and(|x| x == "vnode"))
            .chain(std::iter::once(
                g.path()
                    .join(format!("{}.vgroup", g.file_name().to_string_lossy())),
            ));
        for p in files {
            if mine && p == at_file {
                continue;
            }
            let Ok(f) = crate::sqlite::read(&p) else {
                continue;
            };
            if mine && f.kind().ok() == Some(Kind::Node) {
                nodes.push(f);
            } else {
                elsewhere.push(f);
            }
        }
    }
    let (mut out, reproduced) = edit(
        &group, &nodes, &elsewhere, e, why, by, key, at, mounted_by, programme,
    )?;
    out.meta.insert("written_by_app".into(), app.to_string());
    // The one comparison, as late as can be before the write.
    let again = std::fs::read(&at_file)
        .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", at_file.display())))?;
    if crate::copied::digest(&again) != opened {
        return Ok(Framed::SecondWriter {
            digest: crate::copied::digest(&again),
        });
    }
    let written = crate::sqlite::written(&out, &at_file)?;
    Ok(Framed::Written {
        digest: crate::copied::digest(&written),
        reproduced,
    })
}
