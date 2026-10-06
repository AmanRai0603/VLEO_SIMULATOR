//! The seal: who signed a group's folder, whether each sign-off is still for
//! what is there, and what stands between the folder and its seal.
//!
//! docs/PLAN_1_0.md, phase C: "`web/js/gcheck.js` and the seal rules in
//! `web/js/gseal.js` move into the library. The fingerprint is written twice
//! today, in the page and in `xtask`, and the two have already drifted." The
//! fingerprint is [`format_1::fingerprint`]; these are the rules round it,
//! rule for rule as `gseal.js` has them (`reviewState`, `maySign`,
//! `sealBlockers`), and held to it in `tools/files_check.mjs`:
//!
//! * a sign-off is for the fingerprint it was given for, and is stale the
//!   moment one byte of its scope changes;
//! * the owner signs the whole group; a node is signed by an engineer
//!   members.csv gives it, by name or by `*`, or by the owner;
//! * a folder may be sealed when its checks find no error, and every node and
//!   the group have a current `ok` from somebody who may sign them.
//!
//! This is the seal of a group's folder before 1.0, whose sign-offs are names.
//! From 1.0 a release is signed with keys and checked through the chain
//! ([`crate::chain`]).

use std::collections::BTreeMap;

use crate::csv;
use crate::folder::{self, Finding, Level};
use crate::format_1::{self, Folder};

/// One sign-off in reviews.csv, and whether it is for what is there now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Review {
    pub name: String,
    pub scope: String,
    pub version: String,
    pub fingerprint: String,
    pub date: String,
    pub verdict: String,
    pub note: String,
    /// The fingerprint it was given for is the fingerprint of its scope now.
    pub current: bool,
}

/// One person in members.csv.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub name: String,
    pub role: String,
    pub nodes: String,
}

fn table(folder: &Folder, path: &str) -> Vec<BTreeMap<String, String>> {
    let Some(bytes) = folder.get(path) else {
        return Vec::new();
    };
    let t = csv::read(&String::from_utf8_lossy(bytes));
    t.rows
        .iter()
        .map(|r| {
            t.head
                .iter()
                .enumerate()
                .map(|(k, h)| {
                    (
                        csv::name_of(h).to_string(),
                        r.get(k).cloned().unwrap_or_default(),
                    )
                })
                .collect()
        })
        .collect()
}

fn get(r: &BTreeMap<String, String>, k: &str) -> String {
    r.get(k).cloned().unwrap_or_default()
}

/// The fingerprint of a scope: `group` is the whole folder, anything else one
/// node's folder.
pub fn fingerprint_of(folder: &Folder, scope: &str) -> String {
    format_1::fingerprint(folder, (scope != "group").then_some(scope))
}

/// The people the folder names.
pub fn members(folder: &Folder) -> Vec<Member> {
    table(folder, "members.csv")
        .iter()
        .map(|r| Member {
            name: get(r, "name"),
            role: get(r, "role"),
            nodes: get(r, "nodes"),
        })
        .collect()
}

/// Every sign-off, in the order reviews.csv holds them, each marked current
/// or stale. A sign-off with no scope is taken as for the whole group.
pub fn reviews(folder: &Folder) -> Vec<Review> {
    let mut prints: BTreeMap<String, String> = BTreeMap::new();
    table(folder, "reviews.csv")
        .iter()
        .map(|r| {
            let scope = get(r, "scope");
            let of = if scope.is_empty() { "group" } else { &scope };
            let now = prints
                .entry(of.to_string())
                .or_insert_with(|| fingerprint_of(folder, of))
                .clone();
            let fingerprint = get(r, "fingerprint");
            Review {
                current: fingerprint == now,
                name: get(r, "name"),
                scope,
                version: get(r, "version"),
                fingerprint,
                date: get(r, "date"),
                verdict: get(r, "verdict"),
                note: get(r, "note"),
            }
        })
        .collect()
}

/// Whether a person may sign a scope: the owner signs the group; a node's
/// engineers (or `*`), or the owner, sign a node.
pub fn may_sign(members: &[Member], name: &str, scope: &str) -> bool {
    let Some(m) = members.iter().find(|m| m.name == name) else {
        return false;
    };
    if scope == "group" {
        return m.role == "owner";
    }
    m.role == "owner" || m.nodes.split_whitespace().any(|n| n == "*" || n == scope)
}

/// What still stands between a folder and its seal, in the page's words:
/// the errors its checks find, then every node and the group without a
/// current `ok` from somebody who may sign it. Empty when it may be sealed.
pub fn blockers(folder: &Folder, findings: &[Finding]) -> Vec<String> {
    let mut out = Vec::new();
    let errors = findings.iter().filter(|f| f.level == Level::Error).count();
    if errors > 0 {
        out.push(format!("{errors} error(s) in the checks"));
    }
    let people = members(folder);
    let signed = reviews(folder);
    let ok = |scope: &str| {
        signed.iter().any(|r| {
            r.scope == scope && r.verdict == "ok" && r.current && may_sign(&people, &r.name, scope)
        })
    };
    for id in folder::order(folder) {
        if !ok(&id) {
            out.push(format!(
                "{id} has no current sign-off from its node engineer"
            ));
        }
    }
    if !ok("group") {
        out.push("the group has no current sign-off from its owner".into());
    }
    out
}

/// A folder's seal as the page shows it: the fingerprint of the group and of
/// each node, every sign-off, and what stands in the way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    /// `group` first, then each node in order, with its fingerprint now.
    pub scopes: Vec<(String, String)>,
    pub reviews: Vec<Review>,
    pub blockers: Vec<String>,
}

/// The seal of a folder, its checks made against the pattern.
pub fn state(folder: &Folder, spec: &folder::Spec) -> State {
    let found = folder::check(folder, spec);
    let mut scopes = vec![("group".to_string(), fingerprint_of(folder, "group"))];
    for id in folder::order(folder) {
        let fp = fingerprint_of(folder, &id);
        scopes.push((id, fp));
    }
    State {
        scopes,
        reviews: reviews(folder),
        blockers: blockers(folder, &found),
    }
}

/// A sealed release's files are the ones that were sealed: the fingerprint of
/// the whole folder is the one the seal recorded. Nothing is taken from a
/// release changed after it was signed.
pub fn check_sealed(folder: &Folder, sealed: &str) -> Result<String, String> {
    let have = fingerprint_of(folder, "group");
    if have == sealed {
        Ok(have)
    } else {
        Err(format!(
            "the files are not the ones that were sealed: they give the fingerprint {have}, \
             the seal says {sealed}. Nothing is taken from a release changed after it was signed"
        ))
    }
}
