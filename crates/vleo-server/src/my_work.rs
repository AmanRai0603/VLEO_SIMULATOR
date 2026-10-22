//! My work: what needs the person whose key is open, now (docs/OPERATING_1_0.md,
//! section 4; docs/briefs/F1.md, PR 3).
//!
//! The first items the files can say, each read through the chain as every
//! other check is:
//!
//! - a node assigned to them that is not signed as it is now, or has no file;
//! - a group they own with nothing sealed yet;
//! - a file below them the chain refuses, with why (a group that does not open
//!   stops its people's work as surely as a failing node).
//!
//! - a request on a node of a group they own, or own as its owner's deputy,
//!   for a contract change or a breakdown, signed by whoever may write the
//!   node (F3-d); one that does not count, or says it is answered while no
//!   answer is signed, is named as refused.
//!
//! - a preview on the design released now naming a group they answer for,
//!   owner or deputy, that the group has not answered, or answered by a file
//!   that does not count (F6-c): *Answer <group> <version>*, or *Put back*
//!   an answer written in the copy. Previews, their answers and the design
//!   released now count from the drive the program was started on, the copy
//!   adding only a preview the drive does not hold; a preview that does not
//!   open is said to whoever it may name.
//!
//! Nodes failing their checks, issues and decisions join as the stages that
//! make them arrive (F3 to F7).
//!
//! Beside the items, `nodes` lists every node file that is the person's to
//! write, or their deputy's, signed or not, for the Node workspace to open
//! (F3-g1).

use super::*;
use std::collections::BTreeMap;
use vleo_files::chain::{self, Registry};
use vleo_files::meta::Kind as FileKind;
use vleo_files::sqlite;

struct Item {
    kind: &'static str,
    group: String,
    node: String,
    says: String,
}

/// Each node file in `group`'s folder, by its block: read once for a request,
/// however many of the group's nodes are someone's.
fn node_files(database: &Path, group: &str) -> BTreeMap<String, Vec<vleo_files::model::File>> {
    let nodes = database.join("groups").join(group).join("nodes");
    let mut out: BTreeMap<String, Vec<vleo_files::model::File>> = BTreeMap::new();
    for f in std::fs::read_dir(&nodes)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "vnode"))
        .filter_map(|p| sqlite::read(&p).ok())
        .filter(|f| f.kind().ok() == Some(FileKind::Node))
    {
        if let Some(block) = f.meta.get("block_uid").cloned() {
            out.entry(block).or_default().push(f);
        }
    }
    out
}

/// Whether `releases` holds a release: a sealed group release, a `.vleo` file.
/// Anything else beside them (a partial copy, a system's own file) is not one.
fn sealed(releases: &Path) -> bool {
    std::fs::read_dir(releases).is_ok_and(|d| {
        d.flatten().any(|e| {
            e.file_type().is_ok_and(|t| t.is_file())
                && e.path().extension().is_some_and(|x| x == "vleo")
        })
    })
}

/// `GET /v1/my-work`: who is using the program, and what needs them.
pub(super) fn list(ctx: &Ctx) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    let Some((me, public)) = me::with_key(|key, me| (me.clone(), key.public().to_hex())) else {
        j.bool_field("open", false);
        j.str_field(
            "message",
            "Open your key, and My work shows what needs you now.",
        );
        j.raw("}");
        return j.0;
    };
    j.bool_field("open", true);
    j.str_field("name", &me.name);
    j.key("roles").raw("[");
    for (i, (role, from)) in me.roles.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("role", role);
        j.str_field("in", from);
        j.raw("}");
    }
    j.raw("]");
    let database = me::database(ctx);
    let mut yours = Vec::new();
    let items = match me::registries(&database, &copy::dir(ctx), "") {
        Ok(c) => {
            yours = nodes(&me, &public, &c, &database);
            items(&me, &c, &database)
        }
        Err(me::NotRead::Why(why)) => {
            j.str_field("message", &format!("What needs you cannot be read: {why}"));
            Vec::new()
        }
        Err(me::NotRead::FirstUse(..)) => {
            j.str_field(
                "message",
                "This computer has not accepted this database's anchor: open your key again to see it.",
            );
            Vec::new()
        }
    };
    j.key("items").raw("[");
    for (i, it) in items.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("kind", it.kind);
        j.str_field("group", &it.group);
        j.str_field("node", &it.node);
        j.str_field("says", &it.says);
        j.raw("}");
    }
    j.raw("]");
    j.key("nodes").raw("[");
    for (i, n) in yours.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("group", &n.group);
        j.str_field("block", &n.block);
        j.str_field("id", &n.id);
        j.str_field("revision", &n.revision);
        j.str_field("deputy_for", &n.deputy_for);
        j.bool_field("signed", n.signed);
        j.raw("}");
    }
    j.raw("]}");
    j.0
}

/// A node that is the person's to write, as the Node workspace opens it.
struct Yours {
    group: String,
    block: String,
    id: String,
    revision: String,
    /// Whose deputy they write it as, or empty when it is assigned to them.
    deputy_for: String,
    signed: bool,
}

/// Every node file in the database that is this person's to write, as the
/// library holds it (`Registry::may_sign_node`: assigned to them, or to
/// someone whose deputy they are), in a file of the chain that counts their
/// key now, with one file. The Node workspace opens it from here; a node with
/// no file or two is in the items, with why.
fn nodes(me: &chain::Me, public: &str, c: &me::Chain, database: &Path) -> Vec<Yours> {
    let now = results::now_utc();
    let mut out = Vec::new();
    for reg in c.systems.iter().chain(c.groups.iter()) {
        let Some(group) = reg.file.meta.get("group_id") else {
            continue;
        };
        if reg.key(&me.name, public, &now).is_err() {
            continue;
        }
        let mounts: Vec<&str> = reg
            .file
            .mounts
            .iter()
            .map(|m| m.block_uid.as_str())
            .collect();
        let mut blocks: Vec<&str> = reg
            .file
            .assignments
            .iter()
            .map(|a| a.block_uid.as_str())
            .filter(|b| !mounts.contains(b) && reg.may_sign_node(&me.name, b))
            .collect();
        blocks.sort_unstable();
        blocks.dedup();
        let mut files = None;
        for block in blocks {
            let files = files.get_or_insert_with(|| node_files(database, group));
            let [f] = files.get(block).map(Vec::as_slice).unwrap_or(&[]) else {
                continue;
            };
            let named = reg
                .file
                .assignments
                .iter()
                .any(|a| a.block_uid == block && a.person == me.name);
            let deputy_for = if named {
                String::new()
            } else {
                reg.file
                    .people
                    .iter()
                    .find(|p| {
                        p.name == me.name
                            && reg.file.assignments.iter().any(|a| {
                                a.block_uid == block
                                    && !a.person.is_empty()
                                    && a.person == p.deputy_for
                            })
                    })
                    .map_or_else(String::new, |p| p.deputy_for.clone())
            };
            out.push(Yours {
                group: group.clone(),
                block: block.to_string(),
                id: f
                    .blocks
                    .iter()
                    .find(|b| b.uid == block)
                    .map_or_else(|| block.to_string(), |b| b.id.clone()),
                revision: f.meta.get("revision").cloned().unwrap_or_default(),
                deputy_for,
                signed: chain::check_node(f, block, reg).is_ok(),
            });
        }
    }
    out
}

fn items(me: &chain::Me, c: &me::Chain, database: &Path) -> Vec<Item> {
    let mut out = Vec::new();
    let opened: Vec<&Registry> = c.systems.iter().chain(c.groups.iter()).collect();
    let group_of = |id: &str| {
        opened
            .iter()
            .find(|r| r.file.meta.get("group_id").map(String::as_str) == Some(id))
            .copied()
    };
    let manages = me
        .roles
        .iter()
        .any(|(r, from)| r == "programme manager" && from == "the programme's file");
    let mounted_by_systems = |g: &str| {
        c.systems
            .iter()
            .any(|s| s.file.mounts.iter().any(|m| m.group_id == g))
    };
    // The files below this person that the chain refuses: theirs if they own
    // the group, have nodes in it, or own the file that mounts it: the
    // programme's file is its manager's (the systems' file among what it
    // mounts), and the systems' file its system engineer's.
    for why in &c.left_out {
        let (group, says) = why.split_once(": ").unwrap_or(("", why.as_str()));
        // Mounted by the systems' file and not started: its owner's step,
        // or their deputy's, said as that and not as a refusal (F6-a).
        let unstarted = group != "systems"
            && mounted_by_systems(group)
            && vleo_files::start::to_start(database, group);
        if unstarted {
            let owner = c
                .systems
                .iter()
                .find_map(|s| chain::owner_of(s, group))
                .unwrap_or("");
            let starts =
                !owner.is_empty() && (me.name == owner || me.deputy_for.iter().any(|d| d == owner));
            if starts {
                out.push(Item {
                    kind: "start",
                    group: group.to_string(),
                    node: String::new(),
                    says: format!(
                        "Start {group}: the systems' file mounts it for {owner}, and its file is yours to begin, in the System workspace"
                    ),
                });
                continue;
            }
            if me.owns.iter().any(|g| g == "systems") {
                out.push(Item {
                    kind: "not started",
                    group: group.to_string(),
                    node: String::new(),
                    says: format!("{group} is mounted for {owner}, who starts its file"),
                });
                continue;
            }
        }
        let mounted_by_programme = c.programme.file.mounts.iter().any(|m| m.group_id == group);
        let mine = me.owns.iter().any(|g| g == group)
            || me.nodes.iter().any(|(g, _)| g == group)
            || (mounted_by_programme && manages)
            || (me.owns.iter().any(|g| g == "systems") && mounted_by_systems(group));
        if mine {
            out.push(Item {
                kind: "refused",
                group: group.to_string(),
                node: String::new(),
                says: format!("{group}'s file is refused, so nothing in it counts yet: {says}"),
            });
        }
    }
    let mut indexed: BTreeMap<String, BTreeMap<String, Vec<vleo_files::model::File>>> =
        BTreeMap::new();
    for (group, block) in &me.nodes {
        let Some(reg) = group_of(group) else { continue };
        let files = indexed
            .entry(group.clone())
            .or_insert_with(|| node_files(database, group));
        match files.get(block).map(Vec::as_slice).unwrap_or(&[]) {
            [] => out.push(Item {
                kind: "no file",
                group: group.clone(),
                node: block.clone(),
                says: format!("{block} is assigned to you in {group}, and has no file yet"),
            }),
            [f] => {
                let id = f
                    .blocks
                    .iter()
                    .find(|b| b.uid == *block)
                    .map_or_else(|| block.clone(), |b| b.id.clone());
                if let Err(e) = chain::check_node(f, block, reg) {
                    out.push(Item {
                        kind: "unsigned",
                        group: group.clone(),
                        node: id.clone(),
                        says: format!(
                            "{id} is yours, and not signed as it is now: {}",
                            e.message()
                        ),
                    });
                }
            }
            _ => out.push(Item {
                kind: "two files",
                group: group.clone(),
                node: block.clone(),
                says: format!(
                    "{block} in {group} has more than one file: a node has one, so which is yours is not guessed. Keep one"
                ),
            }),
        }
    }
    // A refused group's file is already said; that it has nothing sealed
    // would be the same fault twice.
    let refused: Vec<&str> = c
        .left_out
        .iter()
        .filter_map(|w| w.split(':').next())
        .collect();
    for group in me
        .owns
        .iter()
        .filter(|g| g.as_str() != "systems" && !refused.contains(&g.as_str()))
    {
        if !sealed(&database.join("groups").join(group).join("releases")) {
            out.push(Item {
                kind: "nothing sealed",
                group: group.clone(),
                node: String::new(),
                says: format!(
                    "{group} has nothing sealed yet: its first release is yours to seal, once its nodes are signed"
                ),
            });
        }
    }
    // The requests on the nodes of a group this person owns, or owns as the
    // owner's deputy (F3-d): every one, whatever it says of itself. One asked
    // and signed by whoever may write its node, and open, is a request; any
    // other is said as refused, with why, never left out: its status is not
    // signed by anyone yet, so a request said to be answered is not taken as
    // answered.
    let mut answers: Vec<String> = me.owns.clone();
    for r in std::iter::once(&c.programme).chain(c.systems.iter()) {
        for m in &r.file.mounts {
            let deputising = chain::owner_of(r, &m.group_id)
                .is_some_and(|owner| me.deputy_for.iter().any(|d| d == owner));
            if deputising && !answers.contains(&m.group_id) {
                answers.push(m.group_id.clone());
            }
        }
    }
    for group in answers.iter().filter(|g| !refused.contains(&g.as_str())) {
        let Some(reg) = group_of(group) else { continue };
        let files = indexed
            .entry(group.clone())
            .or_insert_with(|| node_files(database, group));
        for (block, held) in files.iter() {
            let [f] = held.as_slice() else {
                if held.iter().any(|f| !f.requests.is_empty()) {
                    out.push(Item {
                        kind: "request refused",
                        group: group.clone(),
                        node: block.clone(),
                        says: format!(
                            "{block} in {group} has more than one file, so the requests in them are not read: a node has one file. Keep one"
                        ),
                    });
                }
                continue;
            };
            for r in &f.requests {
                let id = f
                    .blocks
                    .iter()
                    .find(|b| b.uid == r.scope)
                    .map_or_else(|| r.scope.clone(), |b| b.id.clone());
                let asked = match r.kind.as_str() {
                    "breakdown" => "a breakdown",
                    "contract" => "a contract change",
                    other => other,
                };
                let refusal = |why: String| Item {
                    kind: "request refused",
                    group: group.clone(),
                    node: id.clone(),
                    says: format!(
                        "a request on {id} in {group} does not count, so it is not one: {why}"
                    ),
                };
                out.push(match chain::check_request(f, &r.uid, reg) {
                    Err(e) => refusal(e.message().to_string()),
                    Ok(()) if r.status != "open" || !r.answer.is_empty() => refusal(format!(
                        "{}'s request {} says it is {}, and no answer to it is signed yet",
                        r.raised_by, r.uid, r.status
                    )),
                    Ok(()) => Item {
                        kind: "request",
                        group: group.clone(),
                        node: id.clone(),
                        says: format!(
                            "{} asks for {asked} of {id}, on {}: {}",
                            r.raised_by, r.at, r.body
                        ),
                    },
                });
            }
        }
    }
    // The previews on the design released now that name a group this person
    // answers for, owner or deputy (F6-c): each unanswered, or answered by a
    // file that does not count, is theirs to answer.
    // A preview reaches them on the drive, where the system engineer puts
    // it back, and the design released now is the drive's: the drive the
    // program was started on is read first, then the copy.
    let parents: Vec<&Registry> = std::iter::once(&c.programme)
        .chain(c.systems.iter())
        .collect();
    // A preview counts from the drive the program was started on, where the
    // system engineer puts it back, and so does the design released now; the
    // copy only where there is no drive. A file that does not read or open is
    // said to whoever it may name, never passed over.
    let drive: PathBuf = std::env::var_os("VLEO_DRIVE")
        .filter(|v| !v.is_empty())
        .map_or_else(|| database.to_path_buf(), PathBuf::from);
    let refused_preview = |says: String| Item {
        kind: "preview refused",
        group: String::new(),
        node: String::new(),
        says,
    };
    let now = match vleo_files::design::latest_in(&drive.join("design"), &c.programme) {
        Ok(v) => v,
        Err(e) => {
            if !answers.is_empty() {
                out.push(refused_preview(format!(
                    "the design released now cannot be read, so no preview on it is read: {}",
                    e.message()
                )));
            }
            return out;
        }
    };
    let (listed, unread) = crate::design::previews(&drive, database, &parents, &c.programme);
    if !answers.is_empty() {
        for (_, why) in &unread {
            out.push(refused_preview(format!(
                "integration/ does not read, so a preview naming you may be there unseen: {why}"
            )));
        }
    }
    for l in &listed {
        let p = match &l.opened {
            Ok(p) => p,
            Err(why) => {
                // Whom it names, as it says it and unchecked, or anyone who
                // answers when it does not read at all.
                let says_names: Option<Vec<String>> = l.file.as_ref().map(|f| {
                    f.tables
                        .iter()
                        .filter(|t| t.path == "named.csv")
                        .flat_map(|t| vleo_files::csv::read(&t.csv).rows)
                        .filter_map(|r| r.first().cloned())
                        .collect()
                });
                let mine = says_names.as_ref().map_or(!answers.is_empty(), |n| {
                    n.iter().any(|g| answers.contains(g))
                });
                if mine {
                    out.push(refused_preview(format!(
                        "a preview that may name a group of yours does not count: {why}"
                    )));
                }
                continue;
            }
        };
        if now.as_deref() != Some(p.based_on.as_str()) {
            continue;
        }
        for (g, said, in_copy) in
            crate::design::answered_on(p, &drive, database, &parents, &c.programme)
        {
            if !answers.contains(&g) {
                continue;
            }
            let says = match said {
                vleo_files::design::Answered::Unanswered if in_copy => format!(
                    "Put back {g}'s answer to {} {}: it is in your copy, and the drive does not hold it yet",
                    p.group, p.version
                ),
                vleo_files::design::Answered::Unanswered => format!(
                    "Answer {} {}: its preview on design {} names {g}, whose values it moves. Say fine, or object and why, in the System workspace on {g}",
                    p.group, p.version, p.based_on
                ),
                vleo_files::design::Answered::Refused(why) => format!(
                    "{g}'s answer to {} {} does not count, so {g} has not answered it: {why}",
                    p.group, p.version
                ),
                _ => continue,
            };
            out.push(Item {
                kind: "answer",
                group: g,
                node: String::new(),
                says,
            });
        }
    }
    out
}
