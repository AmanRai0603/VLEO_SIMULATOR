//! A file in today's format, upgraded to this one with nothing dropped.
//!
//! docs/OPERATING_1_0.md, section 16, and docs/PLAN_1_0.md, phase C. A group's
//! file from before 1.0 (format 1, `crate::format_1`) opens in the 1.0
//! application and becomes a format-2 file. Nothing a group wrote is left
//! behind: every row lands in a format-2 table, and what format 2 has no column
//! for is kept in the file's meta under `format_1.`. So the upgrade has an
//! exact inverse, [`to_format_1`], and the proof that nothing was dropped is
//! that the inverse gives back every row of the file that was upgraded, and the
//! folder those rows hold gives the fingerprint the people signed.
//!
//! What maps to what:
//!
//! | format 1 | format 2 |
//! |---|---|
//! | `member` | `person`: an owner is the group's subsystem engineer, an author or a reviewer a node engineer |
//! | `node` | `block`, with one output `port`; its author an `assignment` |
//! | `node.kind` | the block's behaviour and its output's state: computed is a method, achieved; declared is stated, decided; required is stated, allocated; achieved is open, achieved |
//! | `input` | an input `port`, and the `wire` into it |
//! | `doc`, `tbl`, `media` | `text`, `tbl`, `media`; the scope `group` is `file` |
//! | `review` | `signature`: a name and a fingerprint, with no key |
//! | `comment` | `comment`; an issue raised in the group application is an `issue` |
//! | `request` | `request` of a contract |
//! | `change` | `change` |
//!
//! A sign-off from before 1.0 is a name, not a key's signature, so it checks
//! through no chain. The programme manager anchors each release from before
//! 1.0 once, by its fingerprint ([`anchor`]); from then on only key signatures
//! count, and an anchored release checks by [`check_anchor`].

use std::collections::BTreeMap;

use crate::error::{Error, ErrorKind};
use crate::format_1::{self, Folder, Old};
use crate::keys::{PublicKey, Signature, SigningKey};
use crate::meta::{Kind, ReleaseVersion, FORMAT};
use crate::model::{
    Assignment, Block, Change, Comment, File, Issue, Media, Person, Port, Request, SignatureRow,
    Table, Tbl, Text, Wire,
};

/// What the meta of an upgraded file says it came from.
pub const FROM_FORMAT_1: &str = "format 1";

/// The scope format 2 gives the file as a whole, where format 1 said `group`.
pub const FILE: &str = "file";

/// Who upgraded a file, and when. The page has no clock or name of its own to
/// give the library, so the caller says.
pub struct Upgrade<'a> {
    /// The application doing it, as `written_by_app` names it.
    pub app: &'a str,
    /// When, as an ISO 8601 time.
    pub at: &'a str,
}

/// A file upgraded, and what the upgrade says a person should know.
#[derive(Debug)]
pub struct Upgraded {
    pub file: File,
    pub said: Vec<String>,
}

fn malformed(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Malformed, message)
}

fn scope_up(scope: &str) -> String {
    if scope == format_1::GROUP {
        FILE.to_string()
    } else {
        scope.to_string()
    }
}

fn scope_down(scope: &str) -> String {
    if scope == FILE {
        format_1::GROUP.to_string()
    } else {
        scope.to_string()
    }
}

fn kind_up(kind: &str) -> Option<(&'static str, &'static str)> {
    Some(match kind {
        "computed" => ("method", "achieved"),
        "declared" => ("stated", "decided"),
        "required" => ("stated", "allocated"),
        "achieved" => ("open", "achieved"),
        _ => return None,
    })
}

fn kind_down(behaviour: &str, state: &str) -> Option<&'static str> {
    Some(match (behaviour, state) {
        ("method", "achieved") => "computed",
        ("stated", "decided") => "declared",
        ("stated", "allocated") => "required",
        ("open", "achieved") => "achieved",
        _ => return None,
    })
}

fn role_up(role: &str) -> &'static str {
    // Every owner of a branch is the system engineer of that branch: a group's
    // owner is its subsystem engineer. Format 1's authors and reviewers are the
    // people who write and sign nodes.
    if role == "owner" {
        "subsystem engineer"
    } else {
        "node engineer"
    }
}

fn role_down(role: &str) -> &'static str {
    if role == "subsystem engineer" {
        "owner"
    } else {
        "author"
    }
}

/// A value's port type, from what format 1 holds for it: a number when it is
/// one or is empty, text otherwise. Format 1 did not say.
fn port_type(value: &str) -> String {
    if value.trim().is_empty() || value.trim().parse::<f64>().is_ok() {
        "number".into()
    } else {
        "text".into()
    }
}

fn kind_of_file(old: &str) -> Option<Kind> {
    Some(match old {
        "release" => Kind::GroupRelease,
        "structure" => Kind::Group,
        "node" => Kind::Node,
        _ => return None,
    })
}

/// The release before `version`, from the group's own `versions.csv`: the row
/// above this version's.
fn previous_release(old: &Old, version: &str) -> Option<String> {
    let t = old
        .tbls
        .iter()
        .find(|t| t.scope == format_1::GROUP && t.path == "versions.csv")?;
    let (head, rows) = crate::csv::parse(&t.csv);
    let col = head
        .iter()
        .position(|h| crate::csv::name_of(h) == "version")?;
    let at = rows.iter().position(|r| r[col] == version)?;
    let before = rows.get(at.checked_sub(1)?)?;
    ReleaseVersion::parse(&before[col]).map(|v| v.to_string())
}

/// The key a value no column holds is kept under, in the upgraded file's meta.
fn kept(what: &str, of: &str) -> String {
    format!("format_1.{what}.{of}")
}

/// A format-1 file, given as its tables, upgraded. Refused, by name, if it is
/// not a format-1 file there is, if a node's uid is `file` (the scope format 2
/// gives the file itself), if an input names a node of this file followed by
/// a dot (format 2 would read it as one of this file's ports), or if it is a
/// sealed release whose files do not give the fingerprint it was sealed with.
pub fn from_format_1(tables: Vec<Table>, how: &Upgrade) -> Result<Upgraded, Error> {
    let old = Old::from_tables(tables)?;
    let old_kind = old.meta.get("file_kind").cloned().unwrap_or_default();
    let kind = kind_of_file(&old_kind).ok_or_else(|| {
        malformed(format!(
            "the file says it is a {old_kind:?}, which is not a kind of file format 1 has"
        ))
    })?;
    let mut said = Vec::new();
    let group = old.meta.get("group_id").cloned().unwrap_or_default();
    let version = old.meta.get("version").cloned().unwrap_or_default();

    // The seal first: a sealed release that does not give its fingerprint is
    // not what the people signed, and is not upgraded into something it is not.
    let folder = old.folder();
    let sealed = old.meta.get("fingerprint").cloned().unwrap_or_default();
    if kind == Kind::GroupRelease && !sealed.is_empty() {
        let have = format_1::fingerprint(&folder, None);
        if have != sealed {
            return Err(Error::new(
                ErrorKind::Tampered,
                format!(
                    "{group} {version} does not give the fingerprint it was sealed with: its files give {have}, the seal says {sealed}. A sealed release is never edited; it is not upgraded"
                ),
            ));
        }
        said.push(format!(
            "{group} {version}: every file is the one sealed — fingerprint {}…",
            &sealed[..12.min(sealed.len())]
        ));
    }

    if let Some(k) = old.meta.keys().find(|k| k.starts_with("format_1.")) {
        return Err(malformed(format!(
            "the meta key {k:?} is one the upgrade keeps format 1's own values under"
        )));
    }
    let uids: Vec<&str> = old.nodes.iter().map(|n| n.uid.as_str()).collect();
    if uids.contains(&FILE) {
        return Err(malformed(format!(
            "a node's uid is {FILE:?}, which format 2 keeps for the file itself"
        )));
    }
    let output_of: BTreeMap<&str, &str> = old
        .nodes
        .iter()
        .map(|n| (n.uid.as_str(), n.output.as_str()))
        .collect();

    // ── meta: every key kept; what format 2 gives a new value is kept beside it
    let mut f = File {
        meta: old.meta.clone(),
        ..File::default()
    };
    let mut set: Vec<(&str, String)> = vec![
        ("file_kind", kind.name().to_string()),
        ("format", FORMAT.to_string()),
        ("written_by_app", how.app.to_string()),
        ("upgraded_from", FROM_FORMAT_1.to_string()),
        ("upgraded_at", how.at.to_string()),
    ];
    match kind {
        Kind::GroupRelease => {
            let previous = previous_release(&old, &version).unwrap_or_default();
            if previous.is_empty() {
                said.push(format!(
                    "{group} {version}: its versions.csv names no release before it, so the file says it is the group's first"
                ));
            }
            set.push(("previous", previous));
            set.push(("based_on", String::new()));
        }
        Kind::Group => {
            set.push(("writer", old.meta.get("owner").cloned().unwrap_or_default()));
            set.push(("based_on", String::new()));
        }
        Kind::Node => {
            let uid = old.meta.get("node_uid").cloned().unwrap_or_default();
            let node = old.nodes.iter().find(|n| n.uid == uid);
            set.push(("block_uid", uid.clone()));
            // Format 1 counted a node's revisions from 0, as issued; format 2
            // counts them from 1.
            set.push(("revision", node.map_or(1, |n| n.revision + 1).to_string()));
            set.push((
                "contract_version",
                node.map_or(1, |n| n.contract_version).to_string(),
            ));
            set.push(("writer", node.map_or(String::new(), |n| n.author.clone())));
        }
        _ => {}
    }
    let mut added = Vec::new();
    for (key, value) in set {
        match old.meta.get(key) {
            Some(was) if *was != value => {
                f.meta.insert(kept("meta", key), was.clone());
            }
            Some(_) => {}
            None => added.push(key),
        }
        f.meta.insert(key.to_string(), value);
    }
    f.meta.insert("format_1.added".into(), added.join(" "));

    // ── people
    for m in &old.members {
        let role = role_up(&m.role);
        if role_down(role) != m.role {
            f.meta.insert(kept("role", &m.name), m.role.clone());
        }
        f.people.push(Person {
            name: m.name.clone(),
            role: role.to_string(),
            deputy_for: String::new(),
        });
    }

    // ── blocks, their outputs, and who writes each
    for n in &old.nodes {
        let (behaviour, state) = kind_up(&n.kind).ok_or_else(|| {
            malformed(format!(
                "{} is a {:?} node, which format 1 has not",
                n.id, n.kind
            ))
        })?;
        f.blocks.push(Block {
            uid: n.uid.clone(),
            id: n.id.clone(),
            parent_uid: String::new(),
            question: n.question.clone(),
            behaviour: behaviour.into(),
            perspective: String::new(),
            ord: n.ord,
            archived: n.archived,
            contract_version: n.contract_version,
            revision: n.revision,
        });
        f.ports.push(Port {
            block_uid: n.uid.clone(),
            direction: "out".into(),
            name: n.output.clone(),
            port_type: port_type(&n.value),
            unit: n.unit.clone(),
            lower: n.lower.clone(),
            upper: n.upper.clone(),
            state: state.into(),
            value: n.value.clone(),
            ..Port::default()
        });
        if !n.stage.is_empty() {
            f.meta.insert(kept("stage", &n.uid), n.stage.clone());
        }
        let mut authors: Vec<String> = Vec::new();
        for a in format_1::split_authors(&n.author) {
            if !a.is_empty() && !authors.contains(&a) {
                authors.push(a);
            }
        }
        if authors.join(", ") != n.author {
            f.meta.insert(kept("author", &n.uid), n.author.clone());
        }
        for a in authors {
            f.assignments.push(Assignment {
                block_uid: n.uid.clone(),
                person: a,
                contract_version: n.contract_version,
            });
        }
    }

    // ── inputs, and the wire into each
    for i in &old.inputs {
        let from_ref = if let Some(out) = output_of.get(i.source.as_str()) {
            format!("{}.{out}", i.source)
        } else {
            if let Some((head, _)) = i.source.split_once('.') {
                if uids.contains(&head) {
                    return Err(malformed(format!(
                        "an input of {} comes from {:?}, which format 2 would read as a port of this file's node {head}",
                        i.node_uid, i.source
                    )));
                }
            }
            i.source.clone()
        };
        f.ports.push(Port {
            block_uid: i.node_uid.clone(),
            direction: "in".into(),
            name: i.name.clone(),
            symbol: i.symbol.clone(),
            port_type: port_type(&i.dflt),
            unit: i.unit.clone(),
            lower: i.min.clone(),
            upper: i.max.clone(),
            state: "decided".into(),
            value: i.dflt.clone(),
            says: i.says.clone(),
            ord: i.ord,
            ..Port::default()
        });
        f.wires.push(Wire {
            to_block: i.node_uid.clone(),
            to_port: i.name.clone(),
            from_ref,
        });
    }

    // ── texts, tables and media
    f.texts = old
        .docs
        .iter()
        .map(|d| Text {
            scope: scope_up(&d.scope),
            kind: d.kind.clone(),
            body: d.body.clone(),
        })
        .collect();
    f.tables = old
        .tbls
        .iter()
        .map(|t| Tbl {
            scope: scope_up(&t.scope),
            path: t.path.clone(),
            csv: t.csv.clone(),
        })
        .collect();
    f.media = old
        .media
        .iter()
        .map(|m| Media {
            scope: scope_up(&m.scope),
            path: m.path.clone(),
            media_type: m.r#type.clone(),
            sha256: m.sha256.clone(),
            bytes: m.bytes.clone(),
        })
        .collect();

    // ── sign-offs: a name and a fingerprint, with no key
    let id_of: BTreeMap<&str, &str> = old
        .nodes
        .iter()
        .map(|n| (n.uid.as_str(), n.id.as_str()))
        .collect();
    let mut stale = 0;
    for r in &old.reviews {
        let print = if r.scope == format_1::GROUP {
            format_1::fingerprint(&folder, None)
        } else {
            format_1::fingerprint(
                &folder,
                Some(id_of.get(r.scope.as_str()).copied().unwrap_or(&r.scope)),
            )
        };
        if print != r.fingerprint {
            stale += 1;
        }
        f.signatures.push(SignatureRow {
            scope: scope_up(&r.scope),
            revision: r.version.clone(),
            signer: r.name.clone(),
            public_key: String::new(),
            digest: r.fingerprint.clone(),
            signature: String::new(),
            signed_at: r.date.clone(),
            verdict: r.verdict.clone(),
            note: r.note.clone(),
        });
    }
    if !old.reviews.is_empty() {
        said.push(format!(
            "{} sign-off(s) from before 1.0 are names, not keys{}: they count once the programme manager anchors this file by its fingerprint",
            old.reviews.len(),
            if stale > 0 {
                format!(", and {stale} of them were given for content that has changed since")
            } else {
                String::new()
            }
        ));
    }

    // ── comments, and the issues raised among them
    for c in &old.comments {
        if c.scope == format_1::GROUP && c.section == "issue" {
            let place = c
                .body
                .lines()
                .find_map(|l| l.strip_prefix("- where: "))
                .unwrap_or("")
                .trim()
                .to_string();
            f.issues.push(Issue {
                number: f.issues.len() as i64 + 1,
                group_id: group.clone(),
                raised_by: c.author.clone(),
                at: c.at.clone(),
                place,
                what: c.body.clone(),
                evidence: String::new(),
                addressed_to: old.meta.get("owner").cloned().unwrap_or_default(),
                closed_by: if c.resolved != 0 {
                    "resolved before 1.0".into()
                } else {
                    String::new()
                },
            });
        } else {
            f.comments.push(Comment {
                scope: scope_up(&c.scope),
                section: c.section.clone(),
                author: c.author.clone(),
                at: c.at.clone(),
                body: c.body.clone(),
                resolved: c.resolved,
            });
        }
    }
    f.requests = old
        .requests
        .iter()
        .enumerate()
        .map(|(k, r)| Request {
            uid: format!("format-1-{}", k + 1),
            kind: "contract".into(),
            scope: r.node_uid.clone(),
            raised_by: r.author.clone(),
            at: r.at.clone(),
            body: r.body.clone(),
            status: r.status.clone(),
            answer: r.answer.clone(),
        })
        .collect();
    f.changes = old
        .changes
        .iter()
        .map(|c| Change {
            at: c.at.clone(),
            who: c.who.clone(),
            scope: scope_up(&c.scope),
            what: c.what.clone(),
            before: c.before.clone(),
            after: c.after.clone(),
        })
        .collect();

    if old
        .nodes
        .iter()
        .any(|n| n.kind == "required" || n.kind == "achieved")
    {
        said.push(
            "the requirements stay as the group wrote them, in requirements.csv: a closure is written from them by the people who own them, never inferred"
                .into(),
        );
    }
    f.check_values()?;

    // Nothing dropped: the upgraded file gives back every row it was made from,
    // or it is not upgraded.
    if let Some(lost) = differs(&old, &to_format_1(&f)?) {
        return Err(malformed(format!(
            "{group} {version} cannot be upgraded without losing {lost}: it is left as it is"
        )));
    }
    Ok(Upgraded { file: f, said })
}

/// The first part of `a` that `b` does not hold the same, or `None`. Comments
/// are compared as a set, since an issue among them comes back after them.
fn differs(a: &Old, b: &Old) -> Option<String> {
    let sorted = |o: &Old| {
        let mut c = o.comments.clone();
        c.sort_by(|x, y| {
            (&x.scope, &x.section, &x.author, &x.at, &x.body, x.resolved)
                .cmp(&(&y.scope, &y.section, &y.author, &y.at, &y.body, y.resolved))
        });
        c
    };
    let parts: [(&str, bool); 11] = [
        ("its meta", a.meta == b.meta),
        ("its members", a.members == b.members),
        ("its nodes", a.nodes == b.nodes),
        ("its inputs", a.inputs == b.inputs),
        ("its texts", a.docs == b.docs),
        ("its tables", a.tbls == b.tbls),
        ("its media", a.media == b.media),
        ("its sign-offs", a.reviews == b.reviews),
        ("its comments and issues", sorted(a) == sorted(b)),
        ("its requests", a.requests == b.requests),
        ("its changes", a.changes == b.changes),
    ];
    parts
        .iter()
        .find(|(_, same)| !same)
        .map(|(what, _)| what.to_string())
}

/// An upgraded file, given back as the format-1 file it was upgraded from:
/// every row, every meta key. Refused if the file was not upgraded from
/// format 1, or if something written since cannot be said in format 1.
pub fn to_format_1(f: &File) -> Result<Old, Error> {
    if f.meta.get("upgraded_from").map(String::as_str) != Some(FROM_FORMAT_1) {
        return Err(Error::new(
            ErrorKind::WrongKind,
            "this file was not upgraded from format 1",
        ));
    }
    let mut old = Old::default();

    // ── meta
    let added: Vec<&str> = f.meta.get("format_1.added").map_or(Vec::new(), |a| {
        a.split(' ').filter(|k| !k.is_empty()).collect()
    });
    for (k, v) in &f.meta {
        if k.starts_with("format_1.") || added.contains(&k.as_str()) {
            continue;
        }
        let was = f.meta.get(&kept("meta", k)).unwrap_or(v);
        old.meta.insert(k.clone(), was.clone());
    }

    // ── members
    for p in &f.people {
        let role = f
            .meta
            .get(&kept("role", &p.name))
            .cloned()
            .unwrap_or_else(|| role_down(&p.role).to_string());
        old.members.push(format_1::Member {
            name: p.name.clone(),
            role,
        });
    }

    // ── nodes
    let out_of = |uid: &str| {
        f.ports
            .iter()
            .find(|p| p.block_uid == uid && p.direction == "out")
    };
    let mut output_of: BTreeMap<&str, &str> = BTreeMap::new();
    for b in &f.blocks {
        let out = out_of(&b.uid)
            .ok_or_else(|| malformed(format!("{} has no output to give back", b.id)))?;
        output_of.insert(b.uid.as_str(), out.name.as_str());
        let kind = kind_down(&b.behaviour, &out.state).ok_or_else(|| {
            malformed(format!(
                "{} is {} with an output {}, which format 1 cannot say",
                b.id, b.behaviour, out.state
            ))
        })?;
        let author = f
            .meta
            .get(&kept("author", &b.uid))
            .cloned()
            .unwrap_or_else(|| {
                f.assignments
                    .iter()
                    .filter(|a| a.block_uid == b.uid)
                    .map(|a| a.person.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            });
        old.nodes.push(format_1::Node {
            uid: b.uid.clone(),
            id: b.id.clone(),
            question: b.question.clone(),
            kind: kind.into(),
            output: out.name.clone(),
            unit: out.unit.clone(),
            lower: out.lower.clone(),
            upper: out.upper.clone(),
            value: out.value.clone(),
            stage: f
                .meta
                .get(&kept("stage", &b.uid))
                .cloned()
                .unwrap_or_default(),
            author,
            ord: b.ord,
            archived: b.archived,
            contract_version: b.contract_version,
            revision: b.revision,
        });
    }

    // ── inputs
    for p in f.ports.iter().filter(|p| p.direction == "in") {
        let from = f
            .wires
            .iter()
            .find(|w| w.to_block == p.block_uid && w.to_port == p.name)
            .ok_or_else(|| {
                malformed(format!(
                    "the input {} of {} has no wire",
                    p.name, p.block_uid
                ))
            })?;
        let source = match from.from_ref.split_once('.') {
            Some((uid, port)) if output_of.get(uid) == Some(&port) => uid.to_string(),
            _ => from.from_ref.clone(),
        };
        old.inputs.push(format_1::Input {
            node_uid: p.block_uid.clone(),
            ord: p.ord,
            name: p.name.clone(),
            symbol: p.symbol.clone(),
            source,
            unit: p.unit.clone(),
            dflt: p.value.clone(),
            min: p.lower.clone(),
            max: p.upper.clone(),
            says: p.says.clone(),
        });
    }

    // ── texts, tables, media, sign-offs
    old.docs = f
        .texts
        .iter()
        .map(|t| format_1::Doc {
            scope: scope_down(&t.scope),
            kind: t.kind.clone(),
            body: t.body.clone(),
        })
        .collect();
    old.tbls = f
        .tables
        .iter()
        .map(|t| format_1::Tbl {
            scope: scope_down(&t.scope),
            path: t.path.clone(),
            csv: t.csv.clone(),
        })
        .collect();
    old.media = f
        .media
        .iter()
        .map(|m| format_1::Media {
            scope: scope_down(&m.scope),
            path: m.path.clone(),
            r#type: m.media_type.clone(),
            sha256: m.sha256.clone(),
            bytes: m.bytes.clone(),
        })
        .collect();
    for s in &f.signatures {
        if !s.public_key.is_empty() {
            // Signed with a key: made since the upgrade, and not format 1's.
            continue;
        }
        old.reviews.push(format_1::Review {
            name: s.signer.clone(),
            scope: scope_down(&s.scope),
            version: s.revision.clone(),
            fingerprint: s.digest.clone(),
            date: s.signed_at.clone(),
            verdict: s.verdict.clone(),
            note: s.note.clone(),
        });
    }

    // ── comments, issues, requests, changes
    for c in &f.comments {
        old.comments.push(format_1::Comment {
            scope: scope_down(&c.scope),
            section: c.section.clone(),
            author: c.author.clone(),
            at: c.at.clone(),
            body: c.body.clone(),
            resolved: c.resolved,
        });
    }
    for i in &f.issues {
        old.comments.push(format_1::Comment {
            scope: format_1::GROUP.into(),
            section: "issue".into(),
            author: i.raised_by.clone(),
            at: i.at.clone(),
            body: i.what.clone(),
            resolved: i64::from(!i.closed_by.is_empty()),
        });
    }
    old.requests = f
        .requests
        .iter()
        .map(|r| format_1::Request {
            node_uid: r.scope.clone(),
            author: r.raised_by.clone(),
            at: r.at.clone(),
            body: r.body.clone(),
            status: r.status.clone(),
            answer: r.answer.clone(),
        })
        .collect();
    old.changes = f
        .changes
        .iter()
        .map(|c| format_1::Change {
            at: c.at.clone(),
            who: c.who.clone(),
            scope: scope_down(&c.scope),
            what: c.what.clone(),
            before: c.before.clone(),
            after: c.after.clone(),
        })
        .collect();
    Ok(old)
}

/// The folder an upgraded file held before 1.0, and the fingerprint over it.
pub fn pre_1_0_fingerprint(f: &File) -> Result<(Folder, String), Error> {
    let folder = to_format_1(f)?.folder();
    let print = format_1::fingerprint(&folder, None);
    Ok((folder, print))
}

/// What the programme manager signs to anchor a release from before 1.0.
pub fn anchor_message(group: &str, version: &str, fingerprint: &str) -> Vec<u8> {
    let mut m = b"vleo anchor 1\0".to_vec();
    for part in [group, version, fingerprint] {
        m.extend_from_slice(part.as_bytes());
        m.push(0);
    }
    m
}

fn release_of(f: &File) -> Result<(String, String, String), Error> {
    if f.kind()? != Kind::GroupRelease {
        return Err(Error::new(
            ErrorKind::WrongKind,
            format!(
                "this is a {}, not a group release: only a release is anchored",
                f.kind()?
            ),
        ));
    }
    let get = |k: &str| f.meta.get(k).cloned().unwrap_or_default();
    let (group, version, sealed) = (get("group_id"), get("version"), get("fingerprint"));
    let (_, have) = pre_1_0_fingerprint(f)?;
    if have != sealed {
        return Err(Error::new(
            ErrorKind::Tampered,
            format!(
                "{group} {version} gives the fingerprint {have}, not the {sealed} it was sealed with: it is not the release the people signed"
            ),
        ));
    }
    Ok((group, version, sealed))
}

/// The scope an anchor is recorded under, in the programme's file.
pub fn anchor_scope(group: &str) -> String {
    format!("release {group}")
}

/// The programme manager's anchor of a release from before 1.0, by its
/// fingerprint: a signature to keep in the programme's file. Refused unless the
/// release is a group release upgraded from format 1 whose content gives the
/// fingerprint it was sealed with.
pub fn anchor(
    release: &File,
    programme_manager: &str,
    key: &SigningKey,
    at: &str,
) -> Result<SignatureRow, Error> {
    let (group, version, print) = release_of(release)?;
    let signature = key.sign(&anchor_message(&group, &version, &print));
    Ok(SignatureRow {
        scope: anchor_scope(&group),
        revision: version,
        signer: programme_manager.to_string(),
        public_key: key.public().to_hex(),
        digest: print,
        signature: signature.to_hex(),
        signed_at: at.to_string(),
        verdict: "ok".into(),
        note: "a release from before 1.0, anchored by its fingerprint".into(),
    })
}

/// Whether a release from before 1.0 is anchored: its content gives the
/// fingerprint it was sealed with, and among `anchors` is the programme
/// manager's signature, made with `key`, over that fingerprint.
pub fn check_anchor(
    release: &File,
    anchors: &[SignatureRow],
    key: &PublicKey,
) -> Result<(), Error> {
    let (group, version, print) = release_of(release)?;
    let mine = key.to_hex();
    let row = anchors
        .iter()
        .find(|a| {
            a.scope == anchor_scope(&group)
                && a.revision == version
                && a.digest == print
                && a.public_key == mine
        })
        .ok_or_else(|| {
            Error::new(
                ErrorKind::Signature,
                format!(
                    "{group} {version} is from before 1.0 and is not anchored: the programme manager has not signed its fingerprint {}…",
                    &print[..12.min(print.len())]
                ),
            )
        })?;
    key.check(
        &anchor_message(&group, &version, &print),
        &Signature::from_hex(&row.signature)?,
    )
}
