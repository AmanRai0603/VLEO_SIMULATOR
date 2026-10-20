//! Requests for the developer (docs/briefs/F6.md, F6-c, item 9; W14).
//!
//! Something the code cannot do yet is asked by whoever writes the node that
//! waits on it, as a request of the code in its node's file, signed
//! (`chain::ask`, `kind` `code`); it reaches the drive when the work is put
//! back, as any request does. `GET /v1/requests` lists every one in the
//! database the program reads, each said as counting or not, with why, and
//! as answered once `apps/NOTES.md`, the application's release notes, names
//! its uid, whole: the developer answers a request by an application release
//! (W15), and never writes the request's status. The notes are not signed:
//! whoever may write `apps/` on the drive can mark one answered, as the brief
//! proposes (F6's question 55). A node file that does not read, or is not one
//! node's one file, is said, never passed over. The route reads the database
//! the program reads, the working copy when there is one; a design released
//! writes the drive's list, as it is at that moment, as
//! `readable/Requests.csv`, each row naming the design it was listed for.

use super::*;
use vleo_files::chain::{self, Registry};

/// One request of the code, as the database holds it.
pub(super) struct Code {
    pub(super) group: String,
    pub(super) node: String,
    pub(super) uid: String,
    pub(super) asked_by: String,
    pub(super) at: String,
    pub(super) body: String,
    /// Why it does not count, when it does not.
    pub(super) refused: Option<String>,
    /// Whether the application's release notes name it.
    pub(super) answered: bool,
}

/// Whether `notes` names `uid` whole: never as part of a longer uid, so
/// `x-r1` is not named by `x-r10`, nor `band-r1` by `solar_band-r1`.
fn names(notes: &str, uid: &str) -> bool {
    let part = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-';
    notes.match_indices(uid).any(|(i, _)| {
        !notes[..i].chars().next_back().is_some_and(part)
            && !notes[i + uid.len()..].chars().next().is_some_and(part)
    })
}

/// The entries of `dir`, sorted; a folder that is there and cannot be
/// listed, or an entry that cannot be read, is said in `unread` as `shown`.
fn entries(dir: &Path, shown: &str, unread: &mut Vec<String>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    match std::fs::read_dir(dir) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => unread.push(format!("{shown}: it cannot be listed: {e}")),
        Ok(list) => {
            for e in list {
                match e {
                    Ok(e) => out.push(e.path()),
                    Err(e) => unread.push(format!("{shown}: an entry cannot be read: {e}")),
                }
            }
        }
    }
    out.sort();
    out
}

/// Every request of the code in `database`'s node files, by group and node,
/// each held to `chain::check_request` through `c`; and each node file that
/// does not read, or is not one node's one file in its group's folder, with
/// why. A request whose status says anything but open does not count: nobody
/// writes the status of a request of the code.
pub(super) fn of_the_code(database: &Path, c: &me::Chain) -> (Vec<Code>, Vec<String>) {
    let (mut out, mut unread) = (Vec::new(), Vec::new());
    let notes = match std::fs::read_to_string(database.join("apps/NOTES.md")) {
        Ok(n) => n,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => {
            unread.push(format!(
                "apps/NOTES.md: it does not read, so no request is taken as answered: {e}"
            ));
            String::new()
        }
    };
    let registry = |group: &str| -> Option<&Registry> {
        std::iter::once(&c.programme)
            .chain(c.systems.iter())
            .chain(c.groups.iter())
            .find(|r| r.file.meta.get("group_id").map(String::as_str) == Some(group))
    };
    for dir in entries(&database.join("groups"), "groups", &mut unread)
        .into_iter()
        .filter(|p| p.is_dir())
    {
        let group = dir
            .file_name()
            .map_or(String::new(), |n| n.to_string_lossy().into_owned());
        let folder = format!("groups/{group}/nodes");
        // Each node's files, by its block: a node has one.
        let mut by_block: BTreeMap<String, Vec<(String, vleo_files::model::File)>> =
            BTreeMap::new();
        for p in entries(&dir.join("nodes"), &folder, &mut unread)
            .into_iter()
            .filter(|p| p.extension().is_some_and(|x| x == "vnode"))
        {
            let shown = format!(
                "{folder}/{}",
                p.file_name()
                    .map_or(String::new(), |n| n.to_string_lossy().into_owned())
            );
            match vleo_files::sqlite::read(&p) {
                Err(e) => unread.push(format!("{shown}: {}", e.message())),
                Ok(f) if f.kind().ok() != Some(vleo_files::meta::Kind::Node) => {
                    unread.push(format!("{shown}: it is not a node's file"))
                }
                Ok(f) if f.meta.get("group_id").map(String::as_str) != Some(group.as_str()) => {
                    unread.push(format!("{shown}: it is not {group}'s node's file"))
                }
                Ok(f) => {
                    let block = f.meta.get("block_uid").cloned().unwrap_or_default();
                    by_block.entry(block).or_default().push((shown, f));
                }
            }
        }
        for (block, held) in by_block {
            let [(_, f)] = held.as_slice() else {
                let names: Vec<&str> = held.iter().map(|(n, _)| n.as_str()).collect();
                unread.push(format!(
                    "{folder}: {block} has more than one file ({}), so the requests in them are not read: a node has one file",
                    names.join(", ")
                ));
                continue;
            };
            for r in f.requests.iter().filter(|r| r.kind == "code") {
                let node = f
                    .blocks
                    .iter()
                    .find(|b| b.uid == r.scope)
                    .map_or_else(|| r.scope.clone(), |b| b.id.clone());
                let refused = match registry(&group) {
                    None => {
                        let why = c
                            .left_out
                            .iter()
                            .find_map(|l| l.strip_prefix(&format!("{group}: ")))
                            .unwrap_or("it is not in the chain");
                        Some(format!(
                            "{group}'s file does not open through the chain, so no request in its folder counts: {why}"
                        ))
                    }
                    Some(reg) => match chain::check_request(f, &r.uid, reg) {
                        Err(e) => Some(e.message().to_string()),
                        Ok(()) if r.status != "open" || !r.answer.is_empty() => Some(format!(
                            "it says it is {}, and nobody writes the status of a request of the code: the developer answers it by naming it in apps/NOTES.md",
                            r.status
                        )),
                        Ok(()) => None,
                    },
                };
                out.push(Code {
                    group: group.clone(),
                    node,
                    uid: r.uid.clone(),
                    asked_by: r.raised_by.clone(),
                    at: r.at.clone(),
                    body: r.body.clone(),
                    refused,
                    answered: names(&notes, &r.uid),
                });
            }
        }
    }
    (out, unread)
}

/// The requests of the code as `readable/Requests.csv` holds them, listed
/// when `design` was released.
pub(super) fn csv(codes: &[Code], unread: &[String], design: &str) -> String {
    let mut rows: Vec<Vec<String>> = codes
        .iter()
        .map(|c| {
            vec![
                design.to_string(),
                c.group.clone(),
                c.node.clone(),
                c.uid.clone(),
                c.asked_by.clone(),
                c.at.clone(),
                c.body.clone(),
                match &c.refused {
                    None => "counts".to_string(),
                    Some(why) => format!("does not count: {why}"),
                },
                if c.answered {
                    "named in apps/NOTES.md"
                } else {
                    "open"
                }
                .to_string(),
            ]
        })
        .collect();
    for u in unread {
        rows.push(vec![
            design.to_string(),
            u.strip_prefix("groups/")
                .and_then(|g| g.split('/').next())
                .unwrap_or("")
                .to_string(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            format!("not read: {u}"),
            String::new(),
        ]);
    }
    vleo_files::csv::write(
        &[
            "design", "group", "node", "request", "asked_by", "at", "says", "counts", "answered",
        ],
        &rows,
    )
}

/// `GET /v1/requests`: every request of the code in the database the
/// program reads, named as `database`.
pub(super) fn list(ctx: &Ctx) -> String {
    let database = me::database(ctx);
    let c =
        match me::registries(&database, &copy::dir(ctx), "") {
            Ok(c) => c,
            Err(me::NotRead::FirstUse(..)) => return me::refusal(
                "this computer has not accepted this database's anchor: open your key on it first",
            ),
            Err(me::NotRead::Why(why)) => return me::refusal(&why),
        };
    let (codes, unread) = of_the_code(&database, &c);
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("database", &database.display().to_string());
    j.key("requests").open_arr();
    for r in &codes {
        j.open_obj();
        j.str_field("group", &r.group);
        j.str_field("node", &r.node);
        j.str_field("uid", &r.uid);
        j.str_field("asked_by", &r.asked_by);
        j.str_field("at", &r.at);
        j.str_field("says", &r.body);
        j.bool_field("counts", r.refused.is_none());
        j.str_field("why_not", r.refused.as_deref().unwrap_or(""));
        j.bool_field("answered", r.answered);
        j.close_obj();
    }
    j.close_arr();
    strings(&mut j, "not_read", &unread);
    let open = codes
        .iter()
        .filter(|r| r.refused.is_none() && !r.answered)
        .count();
    j.str_field(
        "message",
        &format!(
            "{} request{} of the code, {open} open: the developer answers one by an application release whose notes name it",
            codes.len(),
            if codes.len() == 1 { "" } else { "s" }
        ),
    );
    j.raw("}");
    j.0
}

#[cfg(test)]
mod tests {
    use super::names;

    #[test]
    fn a_request_is_answered_only_when_the_notes_name_it_whole() {
        assert!(names("Answers sw_band-r1.", "sw_band-r1"));
        assert!(names("- sw_band-r1\n", "sw_band-r1"));
        assert!(!names("Answers sw_band-r10.", "sw_band-r1"));
        assert!(!names("Answers solar_sw_band-r1.", "sw_band-r1"));
        assert!(names("sw_band-r10, then sw_band-r1", "sw_band-r1"));
        assert!(!names("", "sw_band-r1"));
    }
}
