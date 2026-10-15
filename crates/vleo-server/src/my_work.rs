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
//! Nodes failing their checks, requests, issues, previews and decisions join
//! as the stages that make them arrive (F3 to F7).

use super::*;
use vleo_files::chain::{self, Registry};
use vleo_files::meta::Kind as FileKind;
use vleo_files::sqlite;

struct Item {
    kind: &'static str,
    group: String,
    node: String,
    says: String,
}

/// The node file in `group`'s folder whose block is `block`, if there is one.
fn node_file(
    database: &Path,
    group: &str,
    block: &str,
) -> Option<(String, vleo_files::model::File)> {
    let nodes = database.join("groups").join(group).join("nodes");
    std::fs::read_dir(&nodes)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "vnode"))
        .filter_map(|p| sqlite::read(&p).ok())
        .filter(|f| f.kind().ok() == Some(FileKind::Node))
        .find(|f| f.meta.get("block_uid").map(String::as_str) == Some(block))
        .map(|f| {
            let id = f
                .blocks
                .iter()
                .find(|b| b.uid == block)
                .map_or_else(|| block.to_string(), |b| b.id.clone());
            (id, f)
        })
}

/// `GET /v1/my-work`: who is using the program, and what needs them.
pub(super) fn list(ctx: &Ctx) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    let Some(me) = me::with_key(|_, me| me.clone()) else {
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
    let items = match me::registries(&database, &copy::dir(ctx), "") {
        Ok(c) => items(&me, &c, &database),
        Err(me::NotRead::Why(why)) => {
            j.str_field("message", &format!("Who needs what cannot be read: {why}"));
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
    j.raw("]}");
    j.0
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
    // The files below this person that the chain refuses.
    for why in &c.left_out {
        let (group, says) = why.split_once(": ").unwrap_or(("", why.as_str()));
        let mine = me.owns.iter().any(|g| g == group)
            || me.nodes.iter().any(|(g, _)| g == group)
            || me
                .roles
                .iter()
                .any(|(r, _)| r == "programme manager" || r == "system engineer");
        if mine {
            out.push(Item {
                kind: "refused",
                group: group.to_string(),
                node: String::new(),
                says: format!("{group}'s file is refused, so nothing in it counts yet: {says}"),
            });
        }
    }
    for (group, block) in &me.nodes {
        let Some(reg) = group_of(group) else { continue };
        match node_file(database, group, block) {
            None => out.push(Item {
                kind: "no file",
                group: group.clone(),
                node: block.clone(),
                says: format!("{block} is assigned to you in {group}, and has no file yet"),
            }),
            Some((id, f)) => {
                if let Err(e) = chain::check_node(&f, block, reg) {
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
        }
    }
    for group in me.owns.iter().filter(|g| g.as_str() != "systems") {
        let releases = database.join("groups").join(group).join("releases");
        let sealed = std::fs::read_dir(&releases).is_ok_and(|mut d| d.next().is_some());
        if !sealed {
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
    out
}
