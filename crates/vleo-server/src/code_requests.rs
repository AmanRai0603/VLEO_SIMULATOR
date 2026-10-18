//! Requests for the developer (docs/briefs/F6.md, F6-c, item 9; W14).
//!
//! Something the code cannot do yet is asked by whoever writes the node that
//! waits on it, as a request of the code in its node's file, signed
//! (`chain::ask`, `kind` `code`); it reaches the drive when the work is put
//! back, as any request does. `GET /v1/requests` lists every one in the
//! database the program reads, each said as counting or not, with why, and
//! as answered once `apps/NOTES.md`, the application's release notes, names
//! its uid: the developer answers a request by an application release (W15),
//! and never writes the request's status. A node file that does not read is
//! said, never passed over. The released design writes the same list as
//! `readable/Requests.csv`.

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

/// Every request of the code in `database`'s node files, by group and node,
/// each held to `chain::check_request` through `c`; and each node file that
/// does not read, with why.
pub(super) fn of_the_code(database: &Path, c: &me::Chain) -> (Vec<Code>, Vec<String>) {
    let notes = std::fs::read_to_string(database.join("apps/NOTES.md")).unwrap_or_default();
    let registry = |group: &str| -> Option<&Registry> {
        std::iter::once(&c.programme)
            .chain(c.systems.iter())
            .chain(c.groups.iter())
            .find(|r| r.file.meta.get("group_id").map(String::as_str) == Some(group))
    };
    let (mut out, mut unread) = (Vec::new(), Vec::new());
    let mut groups: Vec<PathBuf> = std::fs::read_dir(database.join("groups"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    groups.sort();
    for dir in groups {
        let group = dir
            .file_name()
            .map_or(String::new(), |n| n.to_string_lossy().into_owned());
        let mut files: Vec<PathBuf> = std::fs::read_dir(dir.join("nodes"))
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "vnode"))
            .collect();
        files.sort();
        for p in files {
            let shown = format!(
                "groups/{group}/nodes/{}",
                p.file_name()
                    .map_or(String::new(), |n| n.to_string_lossy().into_owned())
            );
            let f = match vleo_files::sqlite::read(&p) {
                Ok(f) => f,
                Err(e) => {
                    unread.push(format!("{shown}: {}", e.message()));
                    continue;
                }
            };
            for r in f.requests.iter().filter(|r| r.kind == "code") {
                let node = f
                    .blocks
                    .iter()
                    .find(|b| b.uid == r.scope)
                    .map_or_else(|| r.scope.clone(), |b| b.id.clone());
                let refused = match registry(&group) {
                    None => Some(format!(
                        "{group}'s file does not open through the chain, so no request in its folder counts"
                    )),
                    Some(reg) => chain::check_request(&f, &r.uid, reg)
                        .err()
                        .map(|e| e.message().to_string()),
                };
                out.push(Code {
                    group: group.clone(),
                    node,
                    uid: r.uid.clone(),
                    asked_by: r.raised_by.clone(),
                    at: r.at.clone(),
                    body: r.body.clone(),
                    refused,
                    answered: notes.contains(&r.uid),
                });
            }
        }
    }
    (out, unread)
}

/// The requests of the code as `readable/Requests.csv` holds them.
pub(super) fn csv(codes: &[Code], unread: &[String]) -> String {
    let mut rows: Vec<Vec<String>> = codes
        .iter()
        .map(|c| {
            vec![
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
            String::new(),
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
            "group", "node", "request", "asked_by", "at", "says", "counts", "answered",
        ],
        &rows,
    )
}

/// `GET /v1/requests`: every request of the code in the database the
/// program reads.
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
