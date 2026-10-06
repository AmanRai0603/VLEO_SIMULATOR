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
//! seal covers the node signatures it seals.

use crate::error::{Error, ErrorKind};
use crate::keys::{hex, sha256, PublicKey, Signature, SigningKey};
use crate::meta::Kind;
use crate::model::{Cell, File, Person, SignatureRow, Table};
use crate::upgrade;

/// The scope that is the file as a whole.
pub const FILE: &str = "file";

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
/// ([`FILE`]), or one block by its uid — the block, its ports, the wires into
/// it, its mount, closures, loops, cases, texts, tables and media. A block's
/// children are their own scopes and sign for themselves.
pub fn digest(f: &File, scope: &str) -> Result<String, Error> {
    let mut out = b"vleo content 1\0".to_vec();
    put_cell(&mut out, &Cell::Text(scope.to_string()));
    if scope == FILE {
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

/// The revision a scope is at: a block's own, or the file's.
fn revision_of(f: &File, scope: &str) -> String {
    if scope == FILE {
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

/// The people a file registers and their keys: who may sign what it governs.
#[derive(Clone, Debug)]
pub struct Registry {
    /// What registers them, for messages: "the programme's file", "l3_solar's group file".
    pub from: String,
    pub file: File,
}

impl Registry {
    /// The person `name`, if `public_key` is registered for them here and
    /// counted at `at`: registered by then, and not revoked by then.
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
        if at < k.registered_at.as_str() {
            return Err(refused(
                ErrorKind::Signature,
                format!(
                    "{name} signed at {at} with a key {} registered only from {}",
                    self.from, k.registered_at
                ),
            ));
        }
        if !k.revoked_from.is_empty() && at >= k.revoked_from.as_str() {
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
    fn acts_as(&self, name: &str, role: &str) -> bool {
        self.file
            .people
            .iter()
            .any(|p| p.name == name && p.role == role)
    }

    /// Whether `name` may act for `holder`: it is them, or their deputy.
    fn acts_for(&self, name: &str, holder: &str) -> bool {
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
    let assigned = |name: &str| {
        group
            .file
            .assignments
            .iter()
            .any(|a| a.block_uid == block_uid && group.acts_for(name, &a.person))
    };
    let (ok, why) = counted(f, block_uid, group, assigned);
    if ok.is_empty() {
        return Err(none_counts(&id, &why));
    }
    Ok(())
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
    if release.kind()? != Kind::GroupRelease {
        return Err(refused(
            ErrorKind::WrongKind,
            format!("this is a {}, not a group release", release.kind()?),
        ));
    }
    let mut out = Checked::default();
    if release.meta.get("upgraded_from").map(String::as_str) == Some(upgrade::FROM_FORMAT_1) {
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
            })
            .filter_map(|k| PublicKey::from_hex(&k.public_key).ok());
        let mut why = Vec::new();
        for key in pm {
            match upgrade::check_anchor(release, &programme.file.signatures, &key) {
                Ok(()) => {
                    out.signed = release.blocks.iter().map(|b| b.id.clone()).collect();
                    return Ok(out);
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
    let group = match open_group(release, programme) {
        Ok(g) => g,
        Err(e) => {
            out.refused.push(format!("the seal: {}", e.message()));
            return Ok(out);
        }
    };
    for b in release.blocks.iter().filter(|b| b.archived == 0) {
        match check_node(release, &b.uid, &group) {
            Ok(()) => out.signed.push(b.id.clone()),
            Err(e) => out.refused.push(e.message().to_string()),
        }
    }
    Ok(out)
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
