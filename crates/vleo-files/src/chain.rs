//! What a signature signs, and the chain it checks through.
//!
//! docs/OPERATING_1_0.md, section 14:
//!
//! - **The anchor.** The programme manager's key fingerprint is written in
//!   START HERE. The programme's file is signed by that key, and checks only
//!   if it is.
//! - **The chain.** The programme's file lists the system engineer's, every
//!   subsystem engineer's and every deputy's public key, and which group each
//!   subsystem engineer owns (the group's mount, assigned to them). Each group
//!   file lists its node engineers' keys. A node signature checks against its
//!   group file, a group seal against the programme's file, and a released
//!   design against the system engineer's key there.
//! - **A key someone else may have** is revoked from a date. A signature made
//!   with it from that date does not check; one made before still does.
//! - **Releases from before 1.0** carry names, not keys: they check by the
//!   programme manager's anchor of their fingerprint (`crate::upgrade`).
//!
//! A signature signs a [`digest`]: the SHA-256 of one scope's content, written
//! out canonically so the same content gives the same digest whatever order
//! its rows were stored in. The signature covers the digest and everything the
//! signature row says about itself — the scope, the revision, who, when, the
//! verdict and the note — so none of them can be changed afterwards either.
//! What a file records about itself is not content and is not signed: its
//! comments, issues, requests and changes, a key file's key, and the
//! signatures of the file as a whole. The signatures of its blocks are: a
//! seal covers the node signatures it seals. A request is signed on its own,
//! by whoever asked ([`REQUEST`]): a seal covers that signature, and the
//! request row is the request's own to sign.

use crate::error::{Error, ErrorKind};
use crate::keys::{hex, sha256, PublicKey, Signature, SigningKey};
use crate::meta::Kind;
use crate::model::{Cell, File, Person, PersonKey, SignatureRow, Table};
use crate::upgrade;

/// The scope that is the file as a whole.
pub const FILE: &str = "file";

/// What a request's own signature signs begins with this: `request:<uid>`. A
/// request is signed by whoever asked, apart from the node it is on, so a
/// node engineer asks before their node can be signed (docs/briefs/F3.md,
/// F3-d). A scope that begins so is a request's, never a block's.
pub const REQUEST: &str = "request:";

/// The scope of the signature of the request `uid`.
pub fn request_scope(uid: &str) -> String {
    format!("{REQUEST}{uid}")
}

/// Meta a file writes about its own signing and saving, never part of what is
/// signed: the seal is made over the content, then recorded beside it.
const UNSIGNED_META: &[&str] = &["fingerprint", "sealed_by", "sealed_at", "written_by_app"];

/// Tables that are a file's record of itself, never part of what is signed.
const UNSIGNED_TABLES: &[&str] = &[
    "signature",
    "comment",
    "issue",
    "request",
    "change",
    "locked_key",
];

fn refused(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error::new(kind, message)
}

/// One cell, written so no two different cells write the same bytes.
fn put_cell(out: &mut Vec<u8>, cell: &Cell) {
    let put = |out: &mut Vec<u8>, tag: u8, bytes: &[u8]| {
        out.push(tag);
        out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        out.extend_from_slice(bytes);
    };
    match cell {
        Cell::Null => out.push(0),
        Cell::Int(i) => put(out, 1, &i.to_le_bytes()),
        Cell::Text(s) => put(out, 2, s.as_bytes()),
        Cell::Blob(b) => put(out, 3, b),
    }
}

/// A table's rows that belong to `keep`, written canonically: its name, its
/// columns, then its rows sorted by their bytes.
fn put_table(out: &mut Vec<u8>, t: &Table, keep: impl Fn(&[Cell]) -> bool) {
    let mut rows: Vec<Vec<u8>> = t
        .rows
        .iter()
        .filter(|r| keep(r))
        .map(|r| {
            let mut b = Vec::new();
            for c in r {
                put_cell(&mut b, c);
            }
            b
        })
        .collect();
    rows.sort();
    put_cell(out, &Cell::Text(t.name.clone()));
    put_cell(out, &Cell::Text(t.columns.join(",")));
    out.extend_from_slice(&(rows.len() as u64).to_le_bytes());
    for r in rows {
        out.extend_from_slice(&(r.len() as u64).to_le_bytes());
        out.extend_from_slice(&r);
    }
}

fn column(t: &Table, name: &str) -> Option<usize> {
    t.columns.iter().position(|c| c == name)
}

/// The column of each table that says which block a row belongs to.
fn block_column(table: &str) -> Option<&'static str> {
    Some(match table {
        "block" => "uid",
        "port" | "mount" | "closure" | "loop" | "test_case" => "block_uid",
        "wire" => "to_block",
        "text" | "tbl" | "media" => "scope",
        _ => return None,
    })
}

/// The SHA-256, as hexadecimal, of one scope's content: the whole file
/// ([`FILE`]), one block by its uid — the block, its ports, the wires into
/// it, its mount, closures, loops, cases, texts, tables and media — or one
/// request ([`REQUEST`]): what was asked, by whom, when and of which node. A
/// request's status and answer are its answerer's, and not what the asker
/// signed. A block's children are their own scopes and sign for themselves.
pub fn digest(f: &File, scope: &str) -> Result<String, Error> {
    let mut out = b"vleo content 1\0".to_vec();
    put_cell(&mut out, &Cell::Text(scope.to_string()));
    if let Some(uid) = scope.strip_prefix(REQUEST) {
        let r = the_request(f, uid)?;
        if f.blocks.iter().any(|b| b.uid == scope) {
            return Err(refused(
                ErrorKind::Malformed,
                format!(
                    "{scope:?} names a block and a request: a block's uid never begins {REQUEST:?}"
                ),
            ));
        }
        for part in [&r.uid, &r.kind, &r.scope, &r.raised_by, &r.at, &r.body] {
            put_cell(&mut out, &Cell::Text(part.clone()));
        }
    } else if scope == FILE {
        let meta: Vec<(&String, &String)> = f
            .meta
            .iter()
            .filter(|(k, _)| !UNSIGNED_META.contains(&k.as_str()))
            .collect();
        out.extend_from_slice(&(meta.len() as u64).to_le_bytes());
        for (k, v) in meta {
            put_cell(&mut out, &Cell::Text(k.clone()));
            put_cell(&mut out, &Cell::Text(v.clone()));
        }
        for t in f.to_tables().iter().skip(1) {
            if t.name == "signature" {
                // Every signature but the file's own: a seal covers the
                // signatures of the nodes it seals.
                if let Some(col) = column(t, "scope") {
                    put_table(&mut out, t, |r| r[col] != Cell::Text(FILE.into()));
                }
            } else if !UNSIGNED_TABLES.contains(&t.name.as_str()) {
                put_table(&mut out, t, |_| true);
            }
        }
    } else {
        if !f.blocks.iter().any(|b| b.uid == scope) {
            return Err(refused(
                ErrorKind::Malformed,
                format!("there is no block {scope:?} in this file to sign"),
            ));
        }
        for t in f.to_tables().iter().skip(1) {
            let Some(col) = block_column(&t.name).and_then(|c| column(t, c)) else {
                continue;
            };
            put_table(&mut out, t, |r| r[col] == Cell::Text(scope.to_string()));
        }
    }
    Ok(hex(&sha256(&out)))
}

/// The bytes a signature row's signature is made over.
pub fn message(row: &SignatureRow) -> Vec<u8> {
    let mut m = b"vleo signature 1\0".to_vec();
    for part in [
        &row.scope,
        &row.revision,
        &row.digest,
        &row.signer,
        &row.signed_at,
        &row.verdict,
        &row.note,
    ] {
        m.extend_from_slice(part.as_bytes());
        m.push(0);
    }
    m
}

/// The one request `uid` names in `f`, or why not: none, or more than one. A
/// uid is a request's whole name, so two rows that share one are refused,
/// never read as the first of them.
fn the_request<'a>(f: &'a File, uid: &str) -> Result<&'a crate::model::Request, Error> {
    let mut found = f.requests.iter().filter(|r| r.uid == uid);
    match (found.next(), found.next()) {
        (Some(r), None) => Ok(r),
        (None, _) => Err(refused(
            ErrorKind::Malformed,
            format!("there is no request {uid:?} in this file"),
        )),
        (Some(_), Some(_)) => Err(refused(
            ErrorKind::Malformed,
            format!("more than one request in this file is {uid:?}: a request's uid is its own"),
        )),
    }
}

/// The revision a scope is at: a block's own, or the file's. A request is
/// never edited, so it has one, `1`.
fn revision_of(f: &File, scope: &str) -> String {
    if let Some(uid) = scope.strip_prefix(REQUEST) {
        if f.requests.iter().any(|r| r.uid == uid) {
            "1".into()
        } else {
            String::new()
        }
    } else if scope == FILE {
        f.meta
            .get("revision")
            .or_else(|| f.meta.get("version"))
            .cloned()
            .unwrap_or_default()
    } else {
        f.blocks
            .iter()
            .find(|b| b.uid == scope)
            .map(|b| b.revision.to_string())
            .unwrap_or_default()
    }
}

/// A signature of `scope` as it is now, by `signer` with `key`: a row to add
/// to the file's signatures. `verdict` is `ok` or `changes`.
pub fn sign(
    f: &File,
    scope: &str,
    signer: &str,
    key: &SigningKey,
    at: &str,
    verdict: &str,
    note: &str,
) -> Result<SignatureRow, Error> {
    if verdict != "ok" && verdict != "changes" {
        return Err(refused(
            ErrorKind::Malformed,
            format!("a verdict is ok or changes, not {verdict:?}"),
        ));
    }
    let mut row = SignatureRow {
        scope: scope.to_string(),
        revision: revision_of(f, scope),
        signer: signer.to_string(),
        public_key: key.public().to_hex(),
        digest: digest(f, scope)?,
        signature: String::new(),
        signed_at: at.to_string(),
        verdict: verdict.to_string(),
        note: note.to_string(),
    };
    row.signature = key.sign(&message(&row)).to_hex();
    Ok(row)
}

/// Whether `row` is a signature, by the key it names, of `scope` as it is in
/// `f` now. Refused as stale when the content or revision changed since it was
/// signed, and as not checking when the signature is not that key's.
pub fn check_row(f: &File, row: &SignatureRow) -> Result<(), Error> {
    if row.public_key.is_empty() {
        return Err(refused(
            ErrorKind::Signature,
            format!(
                "{}'s sign-off of {} is a name, not a key's signature: it is from before 1.0",
                row.signer, row.scope
            ),
        ));
    }
    let now = digest(f, &row.scope)?;
    if now != row.digest || revision_of(f, &row.scope) != row.revision {
        return Err(refused(
            ErrorKind::Tampered,
            format!(
                "{}'s signature of {} was given for revision {} and content {}…; it is at revision {} and content {}… now",
                row.signer,
                row.scope,
                row.revision,
                &row.digest[..12.min(row.digest.len())],
                revision_of(f, &row.scope),
                &now[..12]
            ),
        ));
    }
    PublicKey::from_hex(&row.public_key)?
        .check(&message(row), &Signature::from_hex(&row.signature)?)
        .map_err(|_| {
            refused(
                ErrorKind::Signature,
                format!(
                    "{}'s signature of {} does not check against the key it names",
                    row.signer, row.scope
                ),
            )
        })
}

/// A stamp, a day (YYYY-MM-DD) or a moment (YYYY-MM-DDTHH:MM:SSZ), as seconds:
/// a registration, a revocation and a signature are compared as instants,
/// never as text, and a stamp that is neither is refused by name rather than
/// ordered by its letters.
pub fn instant(stamp: &str, what: &str) -> Result<i64, Error> {
    vleo_units::calendar::Civil::parse(stamp)
        .map(|c| c.to_unix())
        .ok_or_else(|| {
            refused(
                ErrorKind::Malformed,
                format!(
                    "{what} is {stamp:?}, which is neither a day (YYYY-MM-DD) nor a moment (YYYY-MM-DDTHH:MM:SSZ)"
                ),
            )
        })
}

/// Whether key `k` is revoked by `when`: it has a revocation, and it is
/// from `when` or before.
pub fn revoked_by(k: &PersonKey, when: i64) -> Result<bool, Error> {
    Ok(!k.revoked_from.is_empty()
        && when
            >= instant(
                &k.revoked_from,
                &format!("{}'s key's revoked_from", k.person),
            )?)
}

/// The people a file registers and their keys: who may sign what it governs.
#[derive(Clone, Debug)]
pub struct Registry {
    /// What registers them, for messages: "the programme's file", "l3_solar's group file".
    pub from: String,
    pub file: File,
}

impl Registry {
    /// The person `name`, if `public_key` is registered for them here and
    /// counted at `at`: registered by then, with the proof its holder made of
    /// it for them (`people::proof_holds`), and not revoked by then.
    pub fn key(&self, name: &str, public_key: &str, at: &str) -> Result<&Person, Error> {
        let person = self
            .file
            .people
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| {
                refused(
                    ErrorKind::Signature,
                    format!("{name} is not registered in {}", self.from),
                )
            })?;
        let k = self
            .file
            .keys
            .iter()
            .find(|k| k.person == name && k.public_key == public_key)
            .ok_or_else(|| {
                refused(
                    ErrorKind::Signature,
                    format!(
                        "the key {name} signed with is not one {} registers for them",
                        self.from
                    ),
                )
            })?;
        // A key counts only with the proof, made with it, that its holder
        // offered it for this person: a key row written by hand signs nothing.
        if !crate::people::proof_holds(name, public_key, &k.proof) {
            return Err(refused(
                ErrorKind::Signature,
                format!(
                    "the key {name} signed with carries no proof in {} that its holder offered it for them: a key without one signs nothing",
                    self.from
                ),
            ));
        }
        let when = instant(at, "the moment it was signed")?;
        if when < instant(&k.registered_at, &format!("{name}'s key's registered_at"))? {
            return Err(refused(
                ErrorKind::Signature,
                format!(
                    "{name} signed at {at} with a key {} registered only from {}",
                    self.from, k.registered_at
                ),
            ));
        }
        if revoked_by(k, when)? {
            return Err(refused(
                ErrorKind::Signature,
                format!(
                    "{name} signed at {at} with a key revoked from {}: the work it covered is signed afresh",
                    k.revoked_from
                ),
            ));
        }
        Ok(person)
    }

    /// Whether `name` holds `role` here, or is registered as the deputy of
    /// someone who does.
    pub fn acts_as(&self, name: &str, role: &str) -> bool {
        self.file
            .people
            .iter()
            .any(|p| p.name == name && p.role == role)
    }

    /// Whether `name` may sign, and write, the node `block_uid`: this group
    /// file assigns it to them, or to someone they are the deputy of. The one
    /// rule of who may sign a node, wherever it is asked.
    pub fn may_sign_node(&self, name: &str, block_uid: &str) -> bool {
        self.file
            .assignments
            .iter()
            .any(|a| a.block_uid == block_uid && self.acts_for(name, &a.person))
    }

    /// Whether `name` may act for `holder`: it is them, or their deputy.
    /// Nobody acts for nobody: an empty holder is held by no one.
    pub(crate) fn acts_for(&self, name: &str, holder: &str) -> bool {
        if holder.is_empty() {
            return false;
        }
        name == holder
            || self
                .file
                .people
                .iter()
                .any(|p| p.name == name && p.deputy_for == holder)
    }
}

/// The current signatures of `scope` with verdict `ok` that check, each by
/// someone `may` allows, through `registry`. The refusals of every other come
/// back with it, so a person reads why each did not count.
fn counted<'a>(
    f: &'a File,
    scope: &str,
    registry: &Registry,
    may: impl Fn(&str) -> bool,
) -> (Vec<&'a SignatureRow>, Vec<String>) {
    let mut ok = Vec::new();
    let mut why = Vec::new();
    for row in f
        .signatures
        .iter()
        .filter(|s| s.scope == scope && s.verdict == "ok")
    {
        let r = check_row(f, row)
            .and_then(|_| {
                // A signature is a person's (AGENTS.md, rule 6): one under an
                // assistant's name never counts, whatever key made it.
                if crate::checks::is_assistant(&row.signer) {
                    return Err(refused(
                        ErrorKind::Signature,
                        format!(
                            "{:?} is an assistant's name, and a signature is a person's",
                            row.signer
                        ),
                    ));
                }
                Ok(())
            })
            .and_then(|_| {
                registry
                    .key(&row.signer, &row.public_key, &row.signed_at)
                    .map(|_| ())
            })
            .and_then(|_| {
                if may(&row.signer) {
                    Ok(())
                } else {
                    Err(refused(
                        ErrorKind::Signature,
                        format!("{} may not sign {scope}", row.signer),
                    ))
                }
            });
        match r {
            Ok(()) => ok.push(row),
            Err(e) => why.push(e.message().to_string()),
        }
    }
    (ok, why)
}

fn none_counts(what: &str, why: &[String]) -> Error {
    refused(
        ErrorKind::Signature,
        if why.is_empty() {
            format!("{what} has no signature")
        } else {
            format!("{what} has no signature that counts: {}", why.join("; "))
        },
    )
}

/// The programme's file, checked against the anchor: signed, as it is now, by
/// its programme manager with the key whose fingerprint START HERE holds.
/// What it registers is then the root every other signature checks through.
pub fn open_programme(programme: File, anchor: &str) -> Result<Registry, Error> {
    if programme.kind()? != Kind::Group {
        return Err(refused(
            ErrorKind::WrongKind,
            format!(
                "the programme's file is a group file, not a {}",
                programme.kind()?
            ),
        ));
    }
    let registry = Registry {
        from: "the programme's file".into(),
        file: programme,
    };
    let (ok, why) = counted(&registry.file, FILE, &registry, |name| {
        registry.acts_as(name, "programme manager")
    });
    let anchored = ok
        .iter()
        .any(|row| PublicKey::from_hex(&row.public_key).is_ok_and(|k| k.fingerprint() == anchor));
    if !anchored {
        let mut why = why;
        if !ok.is_empty() {
            why.push("it is signed, but not with the key START HERE anchors".into());
        }
        return Err(none_counts(
            "the programme's file, by the anchored key,",
            &why,
        ));
    }
    Ok(registry)
}

/// The subsystem engineer who owns `group` in the programme's file: the person
/// assigned the block the group mounts on.
pub fn owner_of<'a>(programme: &'a Registry, group: &str) -> Option<&'a str> {
    let mount = programme.file.mounts.iter().find(|m| m.group_id == group)?;
    programme
        .file
        .assignments
        .iter()
        .find(|a| a.block_uid == mount.block_uid)
        .map(|a| a.person.as_str())
}

/// A group file (or the group file a release carries), checked through the
/// programme: signed as it is by its subsystem engineer, or their deputy,
/// registered in the programme's file. What it registers is then who may
/// sign its nodes.
pub fn open_group(group: &File, programme: &Registry) -> Result<Registry, Error> {
    let id = group.meta.get("group_id").cloned().unwrap_or_default();
    let owner = owner_of(programme, &id).ok_or_else(|| {
        refused(
            ErrorKind::Signature,
            format!("the programme's file mounts no group {id:?}, so nobody may sign it"),
        )
    })?;
    registers_only_its_own(group, &id, programme)?;
    let (ok, why) = counted(group, FILE, programme, |name| {
        programme.acts_as(name, "subsystem engineer") && programme.acts_for(name, owner)
    });
    if ok.is_empty() {
        return Err(none_counts(
            &format!("{id}, by its subsystem engineer {owner},"),
            &why,
        ));
    }
    Ok(Registry {
        from: format!("{id}'s group file"),
        file: group.clone(),
    })
}

/// The file among `files` that mounts `group`, and the group's owner there: the
/// person assigned the block it mounts on. The programme's file mounts the
/// systems' branch, and the systems' file mounts each subsystem's group, so a
/// group's owner is read from whichever file mounts it. A group has one
/// parent: one mounted twice, by no file, or on a block nobody is assigned is
/// refused by name.
pub fn mounted_by<'a>(
    group: &str,
    files: &[&'a Registry],
) -> Result<(&'a Registry, &'a str), Error> {
    let mut found = files.iter().flat_map(|r| {
        r.file
            .mounts
            .iter()
            .filter(move |m| m.group_id == group)
            .map(move |m| (*r, m))
    });
    let (registry, mount) = found.next().ok_or_else(|| {
        refused(
            ErrorKind::Signature,
            format!("no file mounts the group {group:?}, so nobody may sign it"),
        )
    })?;
    if let Some((other, _)) = found.next() {
        return Err(refused(
            ErrorKind::Signature,
            format!(
                "the group {group:?} is mounted twice, in {} and in {}: a group has one parent",
                registry.from, other.from
            ),
        ));
    }
    let owner = registry
        .file
        .assignments
        .iter()
        .find(|a| a.block_uid == mount.block_uid)
        .map(|a| a.person.as_str())
        .ok_or_else(|| {
            refused(
                ErrorKind::Signature,
                format!(
                    "{} mounts {group:?} on a block assigned to nobody, so nobody may sign it",
                    registry.from
                ),
            )
        })?;
    Ok((registry, owner))
}

/// Whether `by`, with the key `public_key` at `at`, may change the group
/// file of `group` as its owner: the person the file that mounts it assigns
/// the mount, or their deputy, with a key the programme's file counts for
/// them then. The one rule of who frames a branch and whom it registers,
/// wherever it is asked (People's changes, F1; the breakdown, F4). Gives the
/// owner.
pub fn may_frame<'a>(
    group: &str,
    by: &str,
    public_key: &str,
    at: &str,
    mounted_by: &'a Registry,
    programme: &Registry,
) -> Result<&'a str, Error> {
    programme.key(by, public_key, at)?;
    let owner = owner_of(mounted_by, group).ok_or_else(|| {
        refused(
            ErrorKind::Signature,
            format!(
                "{} mounts no group {group:?}, so nobody changes its group file",
                mounted_by.from
            ),
        )
    })?;
    if !programme.acts_for(by, owner) {
        return Err(refused(
            ErrorKind::Signature,
            format!(
                "{by} may not change {group}'s group file: its owner {owner}, or their deputy, does"
            ),
        ));
    }
    Ok(owner)
}

/// A group file checked through the file that mounts it: signed, as it is
/// now, by the owner that file assigns, or their deputy, with a key the
/// programme's file registers. The owner is a system engineer or a subsystem
/// engineer there: every owner of a branch is the system engineer of that
/// branch.
pub fn open_group_in(
    group: &File,
    mounted_by: &Registry,
    programme: &Registry,
) -> Result<Registry, Error> {
    let id = group.meta.get("group_id").cloned().unwrap_or_default();
    let owner = owner_of(mounted_by, &id).ok_or_else(|| {
        refused(
            ErrorKind::Signature,
            format!(
                "{} mounts no group {id:?}, so nobody may sign it",
                mounted_by.from
            ),
        )
    })?;
    if !programme.acts_as(owner, "system engineer")
        && !programme.acts_as(owner, "subsystem engineer")
    {
        return Err(refused(
            ErrorKind::Signature,
            format!(
                "{owner} owns {id} in {}, but the programme's file registers them as neither a system engineer nor a subsystem engineer",
                mounted_by.from
            ),
        ));
    }
    registers_only_its_own(group, &id, programme)?;
    let (ok, why) = counted(group, FILE, programme, |name| {
        programme.acts_for(name, owner)
    });
    if ok.is_empty() {
        return Err(none_counts(&format!("{id}, by its owner {owner},"), &why));
    }
    Ok(Registry {
        from: format!("{id}'s group file"),
        file: group.clone(),
    })
}

/// A group file registers its node engineers and nobody else: no other role,
/// nobody the programme's file registers, and no key it registers. One key
/// says one person, and nobody is registered twice, so a group file can
/// neither speak for a person the programme registers nor lock them out.
fn registers_only_its_own(group: &File, id: &str, programme: &Registry) -> Result<(), Error> {
    for p in &group.people {
        if p.role != "node engineer" {
            return Err(refused(
                ErrorKind::Signature,
                format!(
                    "{id}'s group file registers {} as a {}: a group file registers only its node engineers",
                    p.name, p.role
                ),
            ));
        }
        if programme.file.people.iter().any(|q| q.name == p.name) {
            return Err(refused(
                ErrorKind::Signature,
                format!(
                    "{id}'s group file registers {}, whom the programme's file registers: nobody is registered twice",
                    p.name
                ),
            ));
        }
    }
    if let Some(k) = group.keys.iter().find(|k| {
        programme
            .file
            .keys
            .iter()
            .any(|q| q.public_key == k.public_key)
    }) {
        return Err(refused(
            ErrorKind::Signature,
            format!(
                "{id}'s group file registers for {} a key the programme's file registers: one key says one person",
                k.person
            ),
        ));
    }
    Ok(())
}

/// The systems' file, checked through the programme's: it mounts the systems'
/// branch, and the branch is signed by the system engineer the programme's
/// file assigns, or their deputy.
pub fn open_systems(systems: &File, programme: &Registry) -> Result<Registry, Error> {
    let id = systems.meta.get("group_id").cloned().unwrap_or_default();
    let owner = owner_of(programme, &id).ok_or_else(|| {
        refused(
            ErrorKind::Signature,
            format!("the programme's file mounts no group {id:?}, so nobody may sign it"),
        )
    })?;
    if !programme.acts_as(owner, "system engineer") {
        return Err(refused(
            ErrorKind::Signature,
            format!(
                "{owner} owns {id} in the programme's file, but is not registered as its system engineer"
            ),
        ));
    }
    open_group_in(systems, programme, programme)
}

/// Who a key belongs to, as the files register them: their name, the roles
/// they hold and where, whom they stand in for, the groups they own and the
/// nodes assigned to them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Me {
    pub name: String,
    /// Each role and the file that registers it.
    pub roles: Vec<(String, String)>,
    /// Whose deputy they are.
    pub deputy_for: Vec<String>,
    /// The groups whose mount is assigned to them.
    pub owns: Vec<String>,
    /// The nodes assigned to them: each group and the block's uid.
    pub nodes: Vec<(String, String)>,
}

/// The person `public_key` belongs to at `at`, read from the programme's
/// file, the systems' file and the subsystems' group files (`groups`).
///
/// A key counts only in the file that may register its person: the
/// programme's file registers the programme manager, the system engineer,
/// every subsystem engineer and every deputy; a group file registers node
/// engineers, and nobody the programme's file registers (an opened group file
/// holds no other, [`open_group_in`]). A revocation counts in the file that
/// made it: a node engineer revoked by one group still counts in another that
/// has not revoked them, and that group's owner revokes them there. A key
/// nobody registers, one registered for two people, or one revoked everywhere
/// it counted opens nothing, and says why.
///
/// A key counts for a name only with its holder's proof for that name
/// ([`Registry::key`]), so no group's owner registers someone else's key under
/// a second name: a row written without the proof counts for nothing here,
/// and the key still says who it is. Roles come from
/// the files where the key counted; the groups owned from the two files that
/// mount, the programme's and the systems'.
pub fn whoami(
    public_key: &str,
    at: &str,
    programme: &Registry,
    systems: Option<&Registry>,
    groups: &[&Registry],
) -> Result<Me, Error> {
    instant(at, "now")?;
    let below: Vec<&Registry> = systems.into_iter().chain(groups.iter().copied()).collect();
    let all: Vec<&Registry> = std::iter::once(programme)
        .chain(below.iter().copied())
        .collect();
    let in_programme = |name: &str| programme.file.people.iter().any(|p| p.name == name);
    let mut names: Vec<&str> = Vec::new();
    let mut counted_in: Vec<&Registry> = Vec::new();
    let mut why = Vec::new();
    for (i, r) in all.iter().enumerate() {
        for k in r.file.keys.iter().filter(|k| k.public_key == public_key) {
            let may = if i == 0 {
                Ok(())
            } else if in_programme(&k.person) {
                Err(format!(
                    "{} registers {}, whom the programme's file registers: a group file registers only its node engineers",
                    r.from, k.person
                ))
            } else if !r
                .file
                .people
                .iter()
                .any(|p| p.name == k.person && p.role == "node engineer")
            {
                Err(format!(
                    "{} registers {} as other than a node engineer: a group file registers only its node engineers",
                    r.from, k.person
                ))
            } else {
                Ok(())
            };
            match may.and_then(|_| {
                r.key(&k.person, public_key, at)
                    .map_err(|e| e.message().to_string())
            }) {
                Ok(p) => {
                    if !names.contains(&p.name.as_str()) {
                        names.push(&p.name);
                    }
                    if !counted_in.iter().any(|c| std::ptr::eq(*c, *r)) {
                        counted_in.push(r);
                    }
                }
                Err(e) => why.push(e),
            }
        }
    }
    let name = match names.as_slice() {
        [one] => one.to_string(),
        [] if why.is_empty() => {
            return Err(refused(
                ErrorKind::Signature,
                "no file registers this key: whoever registers you (the programme manager, or your subsystem engineer) registers it first",
            ))
        }
        [] => return Err(refused(ErrorKind::Signature, why.join("; "))),
        many => {
            return Err(refused(
                ErrorKind::Signature,
                format!("this key is registered for more than one person ({}), so it says nobody", many.join(", ")),
            ))
        }
    };
    let mut me = Me {
        name: name.clone(),
        ..Me::default()
    };
    for r in &counted_in {
        for p in r.file.people.iter().filter(|p| p.name == name) {
            let role = (p.role.clone(), r.from.clone());
            if !me.roles.contains(&role) {
                me.roles.push(role);
            }
            if !p.deputy_for.is_empty() && !me.deputy_for.contains(&p.deputy_for) {
                me.deputy_for.push(p.deputy_for.clone());
            }
        }
    }
    for r in std::iter::once(programme).chain(systems) {
        for m in &r.file.mounts {
            if owner_of(r, &m.group_id) == Some(name.as_str()) && !me.owns.contains(&m.group_id) {
                me.owns.push(m.group_id.clone());
            }
        }
    }
    for g in counted_in.iter().filter(|r| !std::ptr::eq(**r, programme)) {
        let id = g.file.meta.get("group_id").cloned().unwrap_or_default();
        let mounts: Vec<&str> = g.file.mounts.iter().map(|m| m.block_uid.as_str()).collect();
        for a in g
            .file
            .assignments
            .iter()
            .filter(|a| a.person == name && !mounts.contains(&a.block_uid.as_str()))
        {
            me.nodes.push((id.clone(), a.block_uid.clone()));
        }
    }
    Ok(me)
}

/// A node's signature, checked against its group: a current `ok` by the node
/// engineer the group file registers and assigns to the block, or their
/// deputy. One writer per file: nobody else's signature counts for it.
pub fn check_node(f: &File, block_uid: &str, group: &Registry) -> Result<(), Error> {
    let id = f
        .blocks
        .iter()
        .find(|b| b.uid == block_uid)
        .map(|b| b.id.clone())
        .unwrap_or_else(|| block_uid.to_string());
    let (ok, why) = counted(f, block_uid, group, |name| {
        group.may_sign_node(name, block_uid)
    });
    if ok.is_empty() {
        return Err(none_counts(&id, &why));
    }
    Ok(())
}

/// `f` with the node `block_uid` signed `ok` by `by`, as the day's work
/// (docs/briefs/F3.md, F3-b), and what its checks found — or why not.
/// Refused before signing when `key` is not a key `group` registers for `by`
/// and counts at `at`, and when `by` may not sign the node
/// ([`Registry::may_sign_node`]). Then the signed file is held to the node's
/// own checks (`checks::check_node`: its cases agree with its method, it
/// holds something to sign, no assistant signs it), every failing one named,
/// and its new signature, by itself, to [`check_node`]: what this signs
/// counts through the chain, or nothing is signed.
pub fn sign_node(
    f: &File,
    block_uid: &str,
    by: &str,
    key: &SigningKey,
    at: &str,
    group: &Registry,
) -> Result<(File, crate::checks::Findings), Error> {
    let id = f
        .blocks
        .iter()
        .find(|b| b.uid == block_uid)
        .map(|b| b.id.clone())
        .ok_or_else(|| {
            refused(
                ErrorKind::Malformed,
                format!("the file holds no block {block_uid}, so it cannot be signed"),
            )
        })?;
    group.key(by, &key.public().to_hex(), at)?;
    if !group.may_sign_node(by, block_uid) {
        return Err(refused(
            ErrorKind::Signature,
            format!(
                "{by} is not assigned {id} in {}, nor the deputy of whoever is: only its node engineer, or their deputy, signs it",
                group.from
            ),
        ));
    }
    let row = sign(f, block_uid, by, key, at, "ok", "")?;
    let mut out = f.clone();
    out.signatures.push(row.clone());
    let found = crate::checks::check_node(&out);
    let refusing: Vec<String> = found
        .errors()
        .map(|e| format!("{}: {}", e.place, e.what))
        .collect();
    if !refusing.is_empty() {
        return Err(refused(
            ErrorKind::Malformed,
            format!(
                "{id} is not signed: its checks refuse it ({}) — {}",
                refusing.len(),
                refusing.join("; ")
            ),
        ));
    }
    let mut only = out.clone();
    only.signatures = vec![row];
    check_node(&only, block_uid, group)?;
    Ok((out, found))
}

/// The kinds of request a node file holds: a contract change (W4) and a
/// breakdown (W13). A request of the code is made in My work (W14).
pub const ASKED_IN_A_NODE: &[&str] = &["contract", "breakdown"];

/// The one rule of a request asked in a node's file: what [`ask`] writes, and
/// all a put-back and My work take. It is the only one of its uid, and its uid
/// is its node's followed by `-r` and a count from 1, as [`ask`] makes it;
/// it is of a contract or a breakdown ([`ASKED_IN_A_NODE`]) and says
/// something; it is on a block the file holds; and it carries a current `ok`
/// signature by the person who asked, made at the moment it says it was
/// asked, with a key the group counts for them, when they may write the node
/// (its node engineer, or their deputy). One writer per file: nobody else's
/// request counts in it. Whether it is still open is its answerer's to say.
pub fn check_request(f: &File, uid: &str, group: &Registry) -> Result<(), Error> {
    let r = the_request(f, uid)?;
    let own = uid
        .strip_prefix(&r.scope)
        .and_then(|n| n.strip_prefix("-r"))
        .and_then(|n| n.parse::<u64>().ok())
        .is_some_and(|n| n >= 1 && uid == format!("{}-r{n}", r.scope));
    if !own {
        return Err(refused(
            ErrorKind::Malformed,
            format!(
                "the request {uid:?} is not named for the node it is on ({}-r1, -r2 …): a request is named as asking names it",
                r.scope
            ),
        ));
    }
    if !ASKED_IN_A_NODE.contains(&r.kind.as_str()) {
        return Err(refused(
            ErrorKind::Malformed,
            format!(
                "the request {uid} is of {:?}: a node's file asks for a contract change or a breakdown",
                r.kind
            ),
        ));
    }
    if r.body.trim().is_empty() {
        return Err(refused(
            ErrorKind::Malformed,
            format!("the request {uid} says nothing"),
        ));
    }
    instant(&r.at, &format!("the moment the request {uid} was asked"))?;
    if !f.blocks.iter().any(|b| b.uid == r.scope) {
        return Err(refused(
            ErrorKind::Malformed,
            format!(
                "the request {uid} is on {:?}, which this file does not hold",
                r.scope
            ),
        ));
    }
    let (ok, why) = counted(f, &request_scope(uid), group, |name| {
        name == r.raised_by && group.may_sign_node(name, &r.scope)
    });
    if ok.is_empty() {
        return Err(none_counts(
            &format!("the request {uid} by {}", r.raised_by),
            &why,
        ));
    }
    if !ok.iter().any(|s| s.signed_at == r.at) {
        return Err(refused(
            ErrorKind::Signature,
            format!(
                "the request {uid} says it was asked at {}, and {} signed it at another moment: a request is signed as it is asked",
                r.at, r.raised_by
            ),
        ));
    }
    Ok(())
}

/// `f` with a request on the node `block_uid` added and signed by `by`, as
/// docs/briefs/F3.md (F3-d) says, and its uid — or why not. Refused before
/// anything is added when `kind` is not one a node file asks
/// ([`ASKED_IN_A_NODE`]), when `body` says nothing, when `key` is not a key
/// `group` registers for `by` and counts at `at`, and when `by` may not write
/// the node. The node itself is not touched: its revision and content, and so
/// every signature of it, are as they were. The new signature, by itself, is
/// then held to [`check_request`].
#[allow(clippy::too_many_arguments)]
pub fn ask(
    f: &File,
    block_uid: &str,
    kind: &str,
    body: &str,
    by: &str,
    key: &SigningKey,
    at: &str,
    group: &Registry,
) -> Result<(File, String), Error> {
    let block = f
        .blocks
        .iter()
        .find(|b| b.uid == block_uid)
        .ok_or_else(|| {
            refused(
                ErrorKind::Malformed,
                format!("the file holds no block {block_uid}, so nothing is asked of it"),
            )
        })?;
    let id = block.id.clone();
    if block.archived != 0 {
        return Err(refused(
            ErrorKind::Malformed,
            format!("{id} is archived: nothing is asked of an archived node"),
        ));
    }
    if group.file.mounts.iter().any(|m| m.block_uid == block_uid) {
        return Err(refused(
            ErrorKind::Malformed,
            format!(
                "{id} is where a group hangs in {}, not a node: a change to a mount is asked of the system engineer (W4)",
                group.from
            ),
        ));
    }
    if !ASKED_IN_A_NODE.contains(&kind) {
        return Err(refused(
            ErrorKind::Malformed,
            format!(
                "a node's file asks for a contract change or a breakdown, and {kind:?} is neither: a request of the code is made in My work"
            ),
        ));
    }
    if body.trim().is_empty() {
        return Err(refused(
            ErrorKind::Malformed,
            format!("the request on {id} says nothing: say what is asked, and why"),
        ));
    }
    group.key(by, &key.public().to_hex(), at)?;
    if !group.may_sign_node(by, block_uid) {
        return Err(refused(
            ErrorKind::Signature,
            format!(
                "{by} is not assigned {id} in {}, nor the deputy of whoever is: only its node engineer, or their deputy, writes a request in its file",
                group.from
            ),
        ));
    }
    let mut n = f.requests.iter().filter(|r| r.scope == block_uid).count() + 1;
    let uid = loop {
        let uid = format!("{block_uid}-r{n}");
        if !f.requests.iter().any(|r| r.uid == uid) {
            break uid;
        }
        n += 1;
    };
    let mut out = f.clone();
    out.requests.push(crate::model::Request {
        uid: uid.clone(),
        kind: kind.to_string(),
        scope: block_uid.to_string(),
        raised_by: by.to_string(),
        at: at.to_string(),
        body: body.to_string(),
        status: "open".into(),
        answer: String::new(),
    });
    let row = sign(&out, &request_scope(&uid), by, key, at, "ok", "")?;
    let mut only = out.clone();
    only.signatures = vec![row.clone()];
    check_request(&only, &uid, group)?;
    out.signatures.push(row);
    Ok((out, uid))
}

/// What checking a release found: every node, and the seal.
#[derive(Debug, Default)]
pub struct Checked {
    /// The blocks whose signature counts.
    pub signed: Vec<String>,
    /// Each block, or the seal, that does not, and why.
    pub refused: Vec<String>,
}

impl Checked {
    pub fn holds(&self) -> bool {
        self.refused.is_empty()
    }
}

/// A group's release, checked through the chain. The seal first: the release
/// as it is, signed by the group's subsystem engineer as the programme's file
/// registers them. Then every node it holds, against the group file the
/// release carries. A release from before 1.0 checks by its anchor instead: its
/// fingerprint signed by the programme manager, kept in the programme's file
/// and covered by the programme manager's signature of it.
pub fn check_release(release: &File, programme: &Registry) -> Result<Checked, Error> {
    checked_release(release, programme, || open_group(release, programme))
}

/// A group's release checked as [`check_release`] does, with its seal checked
/// through the file that mounts the group, `mounted_by`, rather than the
/// programme's: in the design, the systems' file mounts each subsystem's group.
pub fn check_release_in(
    release: &File,
    mounted_by: &Registry,
    programme: &Registry,
) -> Result<Checked, Error> {
    checked_release(release, programme, || {
        open_group_in(release, mounted_by, programme)
    })
}

fn checked_release(
    release: &File,
    programme: &Registry,
    open: impl FnOnce() -> Result<Registry, Error>,
) -> Result<Checked, Error> {
    if release.kind()? != Kind::GroupRelease {
        return Err(refused(
            ErrorKind::WrongKind,
            format!("this is a {}, not a group release", release.kind()?),
        ));
    }
    let mut out = Checked::default();
    if release.meta.get("upgraded_from").map(String::as_str) == Some(upgrade::FROM_FORMAT_1) {
        let scope = upgrade::anchor_scope(release.meta.get("group_id").map_or("", String::as_str));
        let pm = programme
            .file
            .people
            .iter()
            .filter(|p| p.role == "programme manager")
            .flat_map(|p| {
                programme
                    .file
                    .keys
                    .iter()
                    .filter(move |k| k.person == p.name)
            });
        let mut why = Vec::new();
        for k in pm {
            let Ok(key) = PublicKey::from_hex(&k.public_key) else {
                continue;
            };
            match upgrade::check_anchor(release, &programme.file.signatures, &key) {
                Ok(()) => {
                    // A programme manager's key anchors only as any signature
                    // of theirs counts ([`Registry::key`]): registered, with
                    // its holder's proof, and not revoked, when it signed.
                    let tried: Vec<Result<&Person, Error>> = programme
                        .file
                        .signatures
                        .iter()
                        .filter(|a| a.scope == scope && a.public_key == k.public_key)
                        .map(|a| programme.key(&k.person, &k.public_key, &a.signed_at))
                        .collect();
                    if tried.iter().any(Result::is_ok) {
                        out.signed = release.blocks.iter().map(|b| b.id.clone()).collect();
                        return Ok(out);
                    }
                    let said: Vec<String> = tried
                        .into_iter()
                        .filter_map(Result::err)
                        .map(|e| e.message().to_string())
                        .collect();
                    why.push(format!(
                        "{}'s anchor does not count: {}",
                        k.person,
                        said.join("; ")
                    ));
                }
                Err(e) => why.push(e.message().to_string()),
            }
        }
        out.refused.push(if why.is_empty() {
            "the release is from before 1.0, and the programme's file registers no programme manager's key to anchor it".into()
        } else {
            why.join("; ")
        });
        return Ok(out);
    }
    let group = match open() {
        Ok(g) => g,
        Err(e) => {
            out.refused.push(format!("the seal: {}", e.message()));
            return Ok(out);
        }
    };
    for b in release.blocks.iter().filter(|b| b.archived == 0) {
        // A heading is the group's own and no node engineer's to sign: it
        // holds the blocks under it, no assignment names it, and the seal,
        // checked above, vouches for it. Any other block no assignment names
        // computes or decides something nobody's signature counts for.
        if !release.assignments.iter().any(|a| a.block_uid == b.uid) {
            if b.behaviour == "children" {
                out.signed.push(b.id.clone());
            } else {
                out.refused.push(format!(
                    "{} is a {} block that no assignment names, so no node engineer's signature counts for it: only a heading is the seal's own",
                    b.id, b.behaviour
                ));
            }
            continue;
        }
        match check_node(release, &b.uid, &group) {
            Ok(()) => out.signed.push(b.id.clone()),
            Err(e) => out.refused.push(e.message().to_string()),
        }
    }
    Ok(out)
}

/// The seal of `release` that counts through the chain: its signature of
/// the whole file by the group's owner, or their deputy, as `mounted_by`
/// names the owner, with a key the programme's file counts. Who sealed it,
/// and when, are this row's, never the release's unsigned meta.
pub fn seal_of<'a>(
    release: &'a File,
    mounted_by: &Registry,
    programme: &Registry,
) -> Result<&'a SignatureRow, Error> {
    let id = release.meta.get("group_id").cloned().unwrap_or_default();
    let owner = owner_of(mounted_by, &id).ok_or_else(|| {
        refused(
            ErrorKind::Signature,
            format!(
                "{} mounts no group {id:?}, so nobody seals it",
                mounted_by.from
            ),
        )
    })?;
    let (ok, why) = counted(release, FILE, programme, |name| {
        programme.acts_for(name, owner)
    });
    ok.into_iter()
        .next()
        .ok_or_else(|| none_counts(&format!("{id}'s release, by its owner {owner},"), &why))
}

/// A group's answer to another group's release (W7, `crate::answer`),
/// checked through the chain, the one rule wherever an answer is read:
/// an answer file, its meta whole; another group's answer to the release,
/// never the systems' branch's, which decides an objection; saying fine,
/// or object and why, and nothing else; signed as it is by the person it
/// says wrote it, who is the answering group's owner, a subsystem engineer
/// who does not own the release's group, or their deputy, as `mounted_by`,
/// the file that mounts the answering group, names its owner, with a key
/// the programme's file registers.
pub fn check_answer(
    answer: &File,
    mounted_by: &Registry,
    programme: &Registry,
) -> Result<(), Error> {
    if answer.kind()? != Kind::PreviewAnswer {
        return Err(refused(
            ErrorKind::WrongKind,
            format!("this is a {}, not an answer", answer.kind()?),
        ));
    }
    answer.check_meta()?;
    let get = |k: &str| answer.meta.get(k).cloned().unwrap_or_default();
    let (group, g, writer) = (get("group_id"), get("answering_group"), get("writer"));
    for id in [&group, &g] {
        if !crate::breakdown::valid_id(id) {
            return Err(refused(
                ErrorKind::Malformed,
                format!("{id:?} is not a group's id"),
            ));
        }
    }
    if group == g {
        return Err(refused(
            ErrorKind::Malformed,
            format!("{g} does not answer its own release: an answer is another group's"),
        ));
    }
    if g == "systems" {
        return Err(refused(
            ErrorKind::Malformed,
            "the systems' branch decides an objection to a release, and does not answer one: a subsystem's group answers",
        ));
    }
    crate::answer::said(answer)?;
    // Its word, and why, and its signatures: nothing rides along unsigned.
    for t in answer.to_tables() {
        let own = [
            "meta",
            <crate::model::Text as crate::model::Record>::TABLE,
            <SignatureRow as crate::model::Record>::TABLE,
        ];
        if !own.contains(&t.name.as_str()) && !t.rows.is_empty() {
            return Err(refused(
                ErrorKind::Malformed,
                format!(
                    "the answer holds {} rows: an answer holds its word and why, and nothing else",
                    t.name
                ),
            ));
        }
    }
    let owner = owner_of(mounted_by, &g).ok_or_else(|| {
        refused(
            ErrorKind::Signature,
            format!(
                "{} mounts no group {g:?}, so nobody answers for it",
                mounted_by.from
            ),
        )
    })?;
    if !programme
        .file
        .people
        .iter()
        .any(|p| p.name == owner && p.role == "subsystem engineer")
    {
        return Err(refused(
            ErrorKind::Signature,
            format!("{g}'s owner {owner} is not a subsystem engineer the programme's file registers: a subsystem's owner answers"),
        ));
    }
    if owner_of(mounted_by, &group) == Some(owner) {
        return Err(refused(
            ErrorKind::Signature,
            format!("{owner} owns both {group} and {g}: a release's owner does not answer it for another group"),
        ));
    }
    let (ok, why) = counted(answer, FILE, programme, |name| {
        name == writer && programme.acts_for(name, owner)
    });
    if ok.is_empty() {
        return Err(none_counts(
            &format!("{g}'s answer, written by {writer} and by its owner {owner},"),
            &why,
        ));
    }
    Ok(())
}

/// The columns of a preview's `moved.csv` (F6-c, `design::preview`): each
/// value outside the release's group that moved, its node and the group it
/// hangs under, its answer in the design released now and with the release
/// in it, empty where it refused, and why where it refused.
pub const PREVIEW_MOVED: &[&str] = &[
    "value",
    "node",
    "group",
    "before",
    "after",
    "before_why",
    "after_why",
];

/// The columns of a preview's `named.csv`: each group whose owner answers.
pub const PREVIEW_NAMED: &[&str] = &["group"];

/// Where the preview of `group`'s release `version`, on the released design
/// `design`, is, inside a copy or the drive: beside the answers to that
/// release, and named for the design it is on, since a release may be
/// previewed again on a later one (F6's question 53).
pub fn preview_path(group: &str, version: &str, design: &str) -> String {
    format!("integration/{group}-{version}/preview-{group}-{version}-on-{design}.vleo")
}

/// A preview (F6-c, `design::preview`), checked through the chain, the one
/// rule wherever a preview is read: a preview file, its meta whole; holding
/// the one release it previews, whole, at the place the drive keeps it, its
/// bytes their SHA-256; its `moved.csv` and `named.csv` as a preview writes
/// them, every value in it one that moved, outside the release's group, and
/// every group named one a value moved in, never the release's own, the
/// systems' or the programme's, and each once; nothing else; and signed as
/// it is by the person it says wrote it, the system engineer or their
/// deputy, with a key the programme's file registers. That the release held
/// is the one it says is `design::open_preview`'s, which reads it.
pub fn check_preview(preview: &File, programme: &Registry) -> Result<(), Error> {
    if preview.kind()? != Kind::Preview {
        return Err(refused(
            ErrorKind::WrongKind,
            format!("this is a {}, not a preview", preview.kind()?),
        ));
    }
    preview.check_meta()?;
    let get = |k: &str| preview.meta.get(k).cloned().unwrap_or_default();
    let (group, version, writer) = (get("group_id"), get("version"), get("writer"));
    let bad = |what: String| refused(ErrorKind::Malformed, format!("the preview {what}"));
    if !crate::breakdown::valid_id(&group) {
        return Err(bad(format!("is of {group:?}, which is not a group's id")));
    }
    // Its release, and its two tables, and its signatures: nothing rides
    // along unsigned.
    for t in preview.to_tables() {
        let own = [
            "meta",
            <crate::model::Tbl as crate::model::Record>::TABLE,
            <crate::model::Media as crate::model::Record>::TABLE,
            <SignatureRow as crate::model::Record>::TABLE,
        ];
        if !own.contains(&t.name.as_str()) && !t.rows.is_empty() {
            return Err(bad(format!(
                "holds {} rows: a preview holds its release, what moved and who answers, and nothing else",
                t.name
            )));
        }
    }
    let at = format!("groups/{group}/releases/{group}-{version}.vleo");
    match preview.media.as_slice() {
        [m] if m.scope == FILE && m.path == at => {
            if hex(&sha256(&m.bytes)) != m.sha256 {
                return Err(bad(format!(
                    "holds {at} whose bytes are not the ones it was written with: their SHA-256 differs"
                )));
            }
        }
        _ => {
            return Err(bad(format!(
                "holds {} file{}: a preview holds the one release it previews, whole, as {at}",
                preview.media.len(),
                if preview.media.len() == 1 { "" } else { "s" }
            )))
        }
    }
    let table = |path: &str, head: &[&str]| -> Result<Vec<Vec<String>>, Error> {
        let mut found = preview.tables.iter().filter(|t| t.path == path);
        let (Some(t), None) = (found.next(), found.next()) else {
            return Err(bad(format!("does not hold {path} once")));
        };
        let csv = crate::csv::read(&t.csv);
        if t.scope != FILE || !csv.problems.is_empty() || csv.head != head {
            return Err(bad(format!(
                "holds a {path} that is not as a preview writes it: its columns are {}",
                head.join(", ")
            )));
        }
        Ok(csv.rows)
    };
    if preview.tables.len() != 2 {
        return Err(bad(
            "holds tables beside moved.csv and named.csv: it holds those two".into(),
        ));
    }
    let moved = table("moved.csv", PREVIEW_MOVED)?;
    let named = table("named.csv", PREVIEW_NAMED)?;
    if moved.is_empty() {
        return Err(bad(format!(
            "says nothing moved: a release that moves nothing outside {group} has no preview"
        )));
    }
    let answer = |cell: &str| -> Result<Option<u64>, Error> {
        if cell.is_empty() {
            return Ok(None);
        }
        cell.parse::<f64>()
            .ok()
            .filter(|x| x.is_finite())
            .map(|x| Some(x.to_bits()))
            .ok_or_else(|| {
                bad(format!(
                    "says {cell:?} is an answer, and it is not a number"
                ))
            })
    };
    for row in &moved {
        let (value, node, g) = (&row[0], &row[1], &row[2]);
        let of_node = value == node
            || value
                .strip_prefix(node.as_str())
                .is_some_and(|r| r.starts_with('.'));
        if node.is_empty() || !of_node || g.is_empty() {
            return Err(bad(format!(
                "says {value:?} moved in {node:?}, which is not a node's value"
            )));
        }
        if *g == group {
            return Err(bad(format!(
                "says {value}, of {group} itself, moved: a preview is of what moves outside it"
            )));
        }
        if answer(&row[3])? == answer(&row[4])? {
            return Err(bad(format!(
                "says {value} moved, and its two answers are one"
            )));
        }
    }
    let mut seen: Vec<&str> = Vec::new();
    for row in &named {
        let g = row[0].as_str();
        if !crate::breakdown::valid_id(g) || ["systems", "programme"].contains(&g) || g == group {
            return Err(bad(format!(
                "names {g:?} to answer: a subsystem's group answers another group's release"
            )));
        }
        if seen.contains(&g) {
            return Err(bad(format!("names {g} twice")));
        }
        if !moved.iter().any(|r| r[2] == g) {
            return Err(bad(format!("names {g}, and no value of {g} moved")));
        }
        seen.push(g);
    }
    let (ok, why) = counted(preview, FILE, programme, |name| {
        name == writer && programme.acts_as(name, "system engineer")
    });
    if ok.is_empty() {
        return Err(none_counts(
            &format!(
                "the preview of {group} {version}, written by {writer} as the system engineer,"
            ),
            &why,
        ));
    }
    Ok(())
}

/// A released design, checked through the chain: signed as it is by the
/// system engineer, or their deputy, as the programme's file registers them.
pub fn check_design(design: &File, programme: &Registry) -> Result<(), Error> {
    if design.kind()? != Kind::Design {
        return Err(refused(
            ErrorKind::WrongKind,
            format!("this is a {}, not a released design", design.kind()?),
        ));
    }
    let (ok, why) = counted(design, FILE, programme, |name| {
        programme.acts_as(name, "system engineer")
    });
    if ok.is_empty() {
        let v = design.meta.get("version").cloned().unwrap_or_default();
        return Err(none_counts(
            &format!("design {v}, by the system engineer,"),
            &why,
        ));
    }
    Ok(())
}
