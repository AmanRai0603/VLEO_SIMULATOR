//! The frame, at the system engineer's valve (docs/briefs/F6.md, F6-a; W1,
//! W13; docs/SYSTEM_MODEL.md, section 2).
//!
//! The systems' file says where each subsystem's group hangs and what it is
//! given. Three edits write it, made as every edit to a branch is made
//! (`breakdown::edit`: through the chain, by the branch's owner or their
//! deputy, saying why, signed afresh and opened again):
//!
//! - **mount** a group on a leaf nobody writes: a `mount` row and its owner's
//!   `assignment`, in the systems' file and nowhere else. The leaf may be the
//!   systems' own, or a leaf of a subsystem's branch (W13), whose owner is
//!   then the **valve above** the new group ([`valve_above`]). A block framed
//!   in the systems' file is answered by its children from then on; a node
//!   file, the systems' or a subsystem's, is not touched: it stays the
//!   node's estimate;
//! - **allocate** on a mount framed in the systems' file: an input,
//!   `allocated`, parameter `system`, a value or both ends of a range, and
//!   the `closure` that says which way it binds the output it bounds. The
//!   sense is given, never defaulted: *the design sustains Ap 200* and *the
//!   design needs Ap 200* are the same number and opposite requirements.
//!
//! A mount is read across files ([`mounted_on_any`]): the branch that holds
//! the block a group hangs on does not break it down, move it or remove it.
//!
//! Nothing is computed here: no margin, no budget. A closure is what the
//! engine reads once a group's release answers it (F6-b).
//!
//! - **reallocate** an allocation given ([`reallocate`], F6-c): another value
//!   or range, and the sense of its closure only when one is given, its why
//!   saying which belief it follows from; what decides an objection names
//!   it there (`design::decisions`).
//!
//! Not here: moving or removing a mount, changing what an allocation bounds
//! or its unit, starting the group's file (`start`), issuing and unassigning
//! (F5).

use crate::breakdown::{port_ok, valid_id, Branch, Place};
use crate::chain::Registry;
use crate::error::{Error, ErrorKind};
use crate::meta::Kind;
use crate::model::{Assignment, Block, Closure, File, Mount, Port};

fn refused(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Malformed, message)
}

/// The roles that own a branch: every owner of a branch is the system
/// engineer of that branch.
const OWNERS: &[&str] = &["subsystem engineer", "system engineer"];

/// Where a block is held: the group whose branch holds it, and the block.
struct Held<'a> {
    group: String,
    block: &'a Block,
    place: Place,
    /// The branch that holds it, read as one tree.
    files: Vec<&'a File>,
}

/// The block `uid` in the systems' branch, or in any other branch of the
/// design `elsewhere` holds: a group file or a node file, never a release.
fn held<'a>(branch: &Branch<'a>, elsewhere: &'a [File], uid: &str) -> Option<Held<'a>> {
    if let Some((block, place)) = branch.find(uid) {
        let mut files = vec![branch.group];
        files.extend(branch.nodes.iter());
        return Some(Held {
            group: "systems".into(),
            block,
            place,
            files,
        });
    }
    let (f, block) = elsewhere
        .iter()
        .filter(|f| matches!(f.kind(), Ok(Kind::Group) | Ok(Kind::Node)))
        .find_map(|f| f.blocks.iter().find(|b| b.uid == uid).map(|b| (f, b)))?;
    let group = f.meta.get("group_id").cloned().unwrap_or_default();
    let files: Vec<&File> = elsewhere
        .iter()
        .filter(|x| {
            matches!(x.kind(), Ok(Kind::Group) | Ok(Kind::Node))
                && x.meta.get("group_id") == Some(&group)
        })
        .collect();
    let place = if f.kind().ok() == Some(Kind::Group) {
        Place::Group
    } else {
        Place::Node(0)
    };
    Some(Held {
        group,
        block,
        place,
        files,
    })
}

/// The mount on `uid` in any of `files`, and the file's group: one block
/// carries one group, wherever the mount is written.
pub fn mounted_on_any<'a>(uid: &str, files: &[&'a File]) -> Option<(&'a Mount, String)> {
    files.iter().find_map(|f| {
        f.mounts
            .iter()
            .find(|m| m.block_uid == uid)
            .map(|m| (m, f.meta.get("group_id").cloned().unwrap_or_default()))
    })
}

/// The mount of `group` in the systems' file `out`, on `block`, for `owner`,
/// with the rows it touched as they were, or refused by name with nothing
/// written. `branch` is the systems' branch, `elsewhere` every other file of
/// the design, `programme` the programme's file, which registers the owner.
pub(crate) fn mount(
    branch: &Branch,
    elsewhere: &[File],
    block: &str,
    group: &str,
    owner: &str,
    out: &mut File,
    programme: &Registry,
) -> Result<(String, String), Error> {
    if !valid_id(group) {
        return Err(refused(format!(
            "{group:?} is not a group's id: lower-case words joined by _, starting with a letter"
        )));
    }
    if group == "programme" || group == "systems" {
        return Err(refused(format!(
            "{group} is the {group}'s own branch: it is mounted by the programme's file, not here"
        )));
    }
    let mut all: Vec<&File> = vec![branch.group];
    all.extend(branch.nodes.iter());
    all.extend(elsewhere.iter());
    if let Some(f) = all
        .iter()
        .find(|f| f.mounts.iter().any(|m| m.group_id == group))
    {
        return Err(refused(format!(
            "{group} is mounted already, in {}'s file: a group has one parent",
            f.meta.get("group_id").map_or("another", String::as_str)
        )));
    }
    if elsewhere.iter().any(|f| {
        f.kind().ok() == Some(Kind::Group)
            && f.meta.get("group_id").map(String::as_str) == Some(group)
    }) {
        return Err(refused(format!(
            "groups/{group}/ holds a group file already: a group is mounted before its file is started, and never over one"
        )));
    }
    // Its folder holds nothing yet, or its owner could never start it
    // (`start::start_in_place`): a release or a node file of that group
    // anywhere in the copy is refused here too.
    if let Some(f) = elsewhere
        .iter()
        .find(|f| f.meta.get("group_id").map(String::as_str) == Some(group))
    {
        return Err(refused(format!(
            "the copy holds a {} of {group} already: a group is mounted with nothing of it yet, so its owner starts it in an empty folder",
            f.kind().map_or("file".to_string(), |k| k.to_string())
        )));
    }
    if all.iter().any(|f| f.blocks.iter().any(|b| b.uid == group)) {
        return Err(refused(format!(
            "{group} is a block's uid in the design already: a started group's heading takes its group's id, and an id is never used twice"
        )));
    }
    let h = held(branch, elsewhere, block).ok_or_else(|| {
        refused(format!(
            "no branch holds a block {block:?}: a group is mounted on a leaf of the design"
        ))
    })?;
    let b = h.block;
    if h.group == "programme" {
        return Err(refused(format!(
            "{} is in the programme's branch: a group is mounted on a leaf of the systems' branch or a subsystem's",
            b.id
        )));
    }
    if b.archived != 0 {
        return Err(refused(format!("{} is removed: it is archived", b.id)));
    }
    // A branch's own heading is the branch: a group mounted there would
    // freeze everything framed under it.
    if b.uid == h.group {
        return Err(refused(format!(
            "{} is the heading of {}'s branch: a group is mounted on a leaf under it",
            b.id, h.group
        )));
    }
    // An issued node of the systems' branch is its node engineer's file:
    // the mount's assignment, written in the systems' file, would let the
    // group's owner sign it.
    if h.group == "systems" && h.place != Place::Group {
        return Err(refused(format!(
            "{} is issued, a node file of the systems' branch: its file is its node engineer's, and a group is mounted on a leaf framed in the systems' file",
            b.id
        )));
    }
    if let Some((_, by)) = mounted_on_any(block, &all) {
        return Err(refused(format!(
            "{} carries a mount already, in {by}'s file: one block carries one group",
            b.id
        )));
    }
    let children: Vec<&Block> = h
        .files
        .iter()
        .flat_map(|f| f.blocks.iter())
        .filter(|x| x.parent_uid == block && x.archived == 0)
        .collect();
    if let Some(c) = children.first() {
        return Err(refused(format!(
            "{} holds {}: a group is mounted on a leaf, a block with no children",
            b.id, c.id
        )));
    }
    // Broken down: its own group file's wires answer an output of it.
    let outs: Vec<&str> = h
        .files
        .iter()
        .flat_map(|f| f.ports.iter())
        .filter(|p| p.block_uid == block && p.direction == "out")
        .map(|p| p.name.as_str())
        .collect();
    if h.files
        .iter()
        .flat_map(|f| f.wires.iter())
        .any(|w| w.to_block == block && outs.contains(&w.to_port.as_str()))
    {
        return Err(refused(format!(
            "{} is broken down: its children answer it, and a group is mounted on a leaf",
            b.id
        )));
    }
    if let Some(a) = h
        .files
        .iter()
        .flat_map(|f| f.assignments.iter())
        .find(|a| a.block_uid == block)
    {
        return Err(refused(format!(
            "{} is assigned to {}: a group is mounted on a leaf nobody writes, and it is unassigned first (F5)",
            b.id, a.person
        )));
    }
    if crate::checks::is_assistant(owner) {
        return Err(refused(format!(
            "{owner} is an assistant's name: a group is owned by a person"
        )));
    }
    // Every row the programme's file registers them by: someone's deputy
    // in any of them is never the owner, whichever row comes first.
    let named: Vec<&crate::model::Person> = programme
        .file
        .people
        .iter()
        .filter(|p| p.name == owner)
        .collect();
    if let Some(p) = named.iter().find(|p| !p.deputy_for.is_empty()) {
        return Err(refused(format!(
            "{owner} is {}'s deputy: a deputy acts for an owner and is never the owner a mount names",
            p.deputy_for
        )));
    }
    if !named.iter().any(|p| OWNERS.contains(&p.role.as_str())) {
        return Err(refused(format!(
            "{owner} is not registered in the programme's file as a subsystem engineer or the system engineer: a group's owner is one"
        )));
    }
    let before = rows(out, block);
    out.mounts.push(Mount {
        block_uid: block.to_string(),
        group_id: group.to_string(),
        release: String::new(),
    });
    out.assignments.push(Assignment {
        block_uid: block.to_string(),
        person: owner.to_string(),
        contract_version: b.contract_version.max(1),
    });
    // A block framed in the systems' file is answered by its children from
    // now; a node file anywhere stays its node's estimate, untouched.
    if h.group == "systems" && h.place == Place::Group {
        for x in out.blocks.iter_mut().filter(|x| x.uid == block) {
            x.behaviour = "children".into();
        }
    }
    Ok((block.to_string(), before))
}

/// What is allocated to the group mounted on `block`, framed in the systems'
/// file `out`: the input `port`, bounding `bounds` the way `sense` says.
#[allow(clippy::too_many_arguments)]
pub(crate) fn allocate(
    branch: &Branch,
    block: &str,
    port: &Port,
    bounds: &str,
    sense: &str,
    says: &str,
    out: &mut File,
) -> Result<(String, String), Error> {
    let (b, place) = branch
        .find(block)
        .ok_or_else(|| refused(format!("the systems' branch has no block {block:?}")))?;
    if b.archived != 0 {
        return Err(refused(format!("{} is removed: it is archived", b.id)));
    }
    if !branch.group.mounts.iter().any(|m| m.block_uid == block) {
        return Err(refused(format!(
            "no group is mounted on {}: an allocation is given to a group, on its mount",
            b.id
        )));
    }
    if place != Place::Group {
        return Err(refused(format!(
            "{} is issued, a node file of its own: its ports are its contract (W4), and it changes by a request",
            b.id
        )));
    }
    match sense {
        "" => {
            return Err(refused(format!(
                "say which way {} binds {}: <= keeps it under the allocation, >= makes it reach it, and neither is assumed",
                port.name, bounds
            )))
        }
        "<=" | ">=" => {}
        other => {
            return Err(refused(format!(
                "{other:?} is not a sense: an allocation binds <= or >="
            )))
        }
    }
    let ports = branch.ports(block);
    if !ports
        .iter()
        .any(|p| p.direction == "out" && p.name == bounds)
    {
        return Err(refused(format!(
            "{} has no output {bounds:?} for the allocation to bound",
            b.id
        )));
    }
    if ports.iter().any(|p| p.name == port.name) {
        return Err(refused(format!(
            "{} has a port {} already: an allocation is a port of its own",
            b.id, port.name
        )));
    }
    let mut p = port.clone();
    p.block_uid = block.to_string();
    p.direction = "in".into();
    p.state = "allocated".into();
    p.parameter = "system".into();
    port_ok(&p, &b.id)?;
    let given = |v: &str| !v.trim().is_empty();
    let (value, lower, upper) = (given(&p.value), given(&p.lower), given(&p.upper));
    if lower != upper {
        return Err(refused(format!(
            "{}'s allocation {} gives one end of a range: a range declares both",
            b.id, p.name
        )));
    }
    match (value, lower) {
        (false, false) => {
            return Err(refused(format!(
                "{}'s allocation {} gives no value and no range: it is a value, or both ends of a range",
                b.id, p.name
            )))
        }
        (true, true) => {
            return Err(refused(format!(
                "{}'s allocation {} gives a value and a range: it is one or the other",
                b.id, p.name
            )))
        }
        _ => {}
    }
    // A number's allocation is numbers: what is not one is refused by name,
    // never read as nothing.
    if p.port_type == "number" || p.port_type == "integer" {
        for (what, v) in [
            ("value", &p.value),
            ("lower end", &p.lower),
            ("upper end", &p.upper),
        ] {
            if given(v) && v.trim().parse::<f64>().is_err() {
                return Err(refused(format!(
                    "{}'s allocation {} has the {what} {:?}, which is not a number",
                    b.id, p.name, v
                )));
            }
        }
    }
    let uid = format!("{block}.{}", p.name);
    if out.closures.iter().any(|c| c.uid == uid) {
        return Err(refused(format!(
            "a closure {uid} is in the systems' file already"
        )));
    }
    let before = rows(out, block);
    p.ord = out
        .ports
        .iter()
        .filter(|x| x.block_uid == block)
        .map(|x| x.ord + 1)
        .max()
        .unwrap_or(0);
    out.closures.push(Closure {
        uid,
        block_uid: block.to_string(),
        required_ref: format!("{block}.{}", p.name),
        achieved_ref: format!("{block}.{bounds}"),
        sense: sense.to_string(),
        says: says.trim().to_string(),
    });
    out.ports.push(p);
    Ok((block.to_string(), before))
}

/// The allocation `name` of the group mounted on `block`, framed in the
/// systems' file `out`, given the value `value` or the range `lower` to
/// `upper`, as an allocation is given one; its closure's sense changed to
/// `sense` only when one is given, and kept otherwise (question 48). Its
/// unit, type and what it bounds are not changed here. Refused by name, with
/// nothing written.
#[allow(clippy::too_many_arguments)]
pub(crate) fn reallocate(
    branch: &Branch,
    block: &str,
    name: &str,
    value: &str,
    lower: &str,
    upper: &str,
    sense: &str,
    out: &mut File,
) -> Result<(String, String), Error> {
    let (b, place) = branch
        .find(block)
        .ok_or_else(|| refused(format!("the systems' branch has no block {block:?}")))?;
    if place != Place::Group || !branch.group.mounts.iter().any(|m| m.block_uid == block) {
        return Err(refused(format!(
            "no group is mounted on {} in the systems' file: an allocation is changed on its mount",
            b.id
        )));
    }
    let Some(at) = out.ports.iter().position(|p| {
        p.block_uid == block && p.name == name && p.direction == "in" && p.state == "allocated"
    }) else {
        return Err(refused(format!(
            "{} has no allocation {name:?} to change: an allocation is given first, by allocate",
            b.id
        )));
    };
    let given = |v: &str| !v.trim().is_empty();
    if given(lower) != given(upper) {
        return Err(refused(format!(
            "{}'s allocation {name} is given one end of a range: a range declares both",
            b.id
        )));
    }
    match (given(value), given(lower)) {
        (false, false) => {
            return Err(refused(format!(
                "{}'s allocation {name} is given no value and no range: it is a value, or both ends of a range",
                b.id
            )))
        }
        (true, true) => {
            return Err(refused(format!(
                "{}'s allocation {name} is given a value and a range: it is one or the other",
                b.id
            )))
        }
        _ => {}
    }
    let uid = format!("{block}.{name}");
    let closure = out.closures.iter().position(|c| c.required_ref == uid);
    match sense {
        "" => {}
        "<=" | ">=" if closure.is_some() => {}
        "<=" | ">=" => {
            return Err(refused(format!(
                "{}'s allocation {name} bounds nothing: it has no closure whose sense could change",
                b.id
            )))
        }
        other => {
            return Err(refused(format!(
                "{other:?} is not a sense: an allocation binds <= or >="
            )))
        }
    }
    let mut p = out.ports[at].clone();
    p.value = value.trim().to_string();
    p.lower = lower.trim().to_string();
    p.upper = upper.trim().to_string();
    if p.port_type == "number" || p.port_type == "integer" {
        for (what, v) in [
            ("value", &p.value),
            ("lower end", &p.lower),
            ("upper end", &p.upper),
        ] {
            if given(v) && v.parse::<f64>().is_err() {
                return Err(refused(format!(
                    "{}'s allocation {name} is given the {what} {v:?}, which is not a number",
                    b.id
                )));
            }
        }
    }
    port_ok(&p, &b.id)?;
    let before = rows(out, block);
    if p == out.ports[at] && sense.is_empty() {
        return Err(refused(format!(
            "{}'s allocation {name} is given what it holds already: nothing changes",
            b.id
        )));
    }
    out.ports[at] = p;
    if let (Some(c), false) = (closure, sense.is_empty()) {
        out.closures[c].sense = sense.to_string();
    }
    Ok((block.to_string(), before))
}

/// The rows of the frame on `uid` in `f`, as text, for a `change` row: its
/// mounts, its assignments, its allocations and their closures.
pub(crate) fn rows(f: &File, uid: &str) -> String {
    let mut o = String::new();
    for m in f.mounts.iter().filter(|m| m.block_uid == uid) {
        o.push_str(&format!("mount {:?} on {:?}\n", m.group_id, m.block_uid));
    }
    for a in f.assignments.iter().filter(|a| a.block_uid == uid) {
        o.push_str(&format!("assignment {:?} to {:?}\n", a.block_uid, a.person));
    }
    for p in f
        .ports
        .iter()
        .filter(|p| p.block_uid == uid && p.state == "allocated")
    {
        o.push_str(&format!(
            "allocation {:?}.{:?} value={:?} range={:?}..{:?} unit={:?} parameter={:?}\n",
            p.block_uid, p.name, p.value, p.lower, p.upper, p.unit, p.parameter
        ));
    }
    for c in f.closures.iter().filter(|c| c.block_uid == uid) {
        o.push_str(&format!(
            "closure {:?} {:?} {} {:?}\n",
            c.uid, c.achieved_ref, c.sense, c.required_ref
        ));
    }
    o
}

/// The valve above `group`: the branch that holds the block it is mounted
/// on in the systems' file, and that branch's owner. A group mounted on the
/// systems' own leaf has the system engineer above it; one mounted on a leaf
/// of a subsystem's branch (W13), that subsystem's engineer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Valve {
    pub branch: String,
    pub owner: String,
}

/// The valve above `group`, read from the systems' file `systems` and the
/// design's other files `elsewhere`; `programme` gives the systems' owner.
pub fn valve_above(
    group: &str,
    systems: &File,
    systems_nodes: &[File],
    elsewhere: &[File],
    programme: &Registry,
) -> Option<Valve> {
    let m = systems.mounts.iter().find(|m| m.group_id == group)?;
    let branch = Branch::new(systems, systems_nodes);
    let h = held(&branch, elsewhere, &m.block_uid)?;
    let owner = if h.group == "systems" {
        crate::chain::owner_of(programme, "systems")
    } else {
        systems
            .mounts
            .iter()
            .find(|x| x.group_id == h.group)
            .and_then(|x| {
                systems
                    .assignments
                    .iter()
                    .find(|a| a.block_uid == x.block_uid)
            })
            .map(|a| a.person.as_str())
    }?;
    Some(Valve {
        branch: h.group,
        owner: owner.to_string(),
    })
}
