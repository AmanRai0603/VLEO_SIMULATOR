//! A node's revision saved in place, in the person's working copy
//! (docs/briefs/F3.md, F3-a; W3).
//!
//! A node engineer opens a node file from the copy, edits it, and saves it as
//! the next revision. Two things can go wrong, and each is refused with
//! nothing written:
//!
//! - **A second writer.** The file in the copy is not the one the person
//!   opened: someone, or another program on the same copy, saved it since.
//!   The refusal carries the file as it is now, so both can be shown and the
//!   person keeps one. *Changed* means changed bytes: a file touched and not
//!   changed is not a conflict.
//! - **Not this person's to write.** One writer per file: the file is written
//!   by whoever's key is open, and only when the group file assigns them the
//!   block, or they are the deputy of whoever it does.
//! - **Not this node's to change.** A save changes the node's own rows: its
//!   block, ports, wires, cases, closures, loops, texts, tables, media,
//!   comments and changes. Everything else in the file is as it was: its
//!   group, its kind, its place, its signatures, its requests, and every row
//!   of anyone or anything else. A node file holds one block, in its group's
//!   folder, under its id.
//! - **A request is kept as it was asked.** A request is added only by
//!   asking ([`ask_in_place`], F3-d), signed by whoever asked, and no save
//!   changes or drops one.
//!
//! Otherwise the file is checked as a file (its meta and its values, as every
//! write checks them), its revision rises by one and its writer is the
//! person, and it is written whole: to a file beside it, then renamed over
//! the old one (`sqlite::write`). What its content still lacks is said beside
//! the save, never refused here: a day's work is saved while it is still
//! work. Signing it (F3-b) is where the content must hold.
//!
//! The copy's lock is held from the first read to the rename, so two writers
//! through this library, in one program or two on one computer, never write
//! over each other. A folder kept in step by a sync client is not covered: the
//! lock and the comparison are of this computer's copy.
//!
//! The drive is never written here: work goes back by `put_back` (F3-c).

use std::path::{Path, PathBuf};

use crate::chain::Registry;
use crate::checks::{self, Findings};
use crate::copied;
use crate::error::{Error, ErrorKind};
use crate::meta::Kind;
use crate::model::File;

/// What a save did, or why it did nothing.
#[derive(Debug)]
pub enum Saved {
    /// Written as `revision`. `findings` is what its content still lacks.
    Written {
        revision: u64,
        digest: String,
        findings: Findings,
    },
    /// The file changed in the copy since it was opened: nothing is written.
    /// It is as `now` holds it, last written by `writer` as `revision`.
    SecondWriter {
        now: Box<File>,
        bytes: Vec<u8>,
        writer: String,
        revision: u64,
        digest: String,
    },
}

/// The lock beside a copy: held while one program copies the database or
/// writes the copy, and let go when dropped.
#[derive(Debug)]
pub struct Lock(PathBuf);

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// The copy's lock, taken, or why not: another program holds it. It is the
/// file `.<copy's name>.lock` in the folder the copy is in.
pub fn lock(copy: &Path) -> Result<Lock, Error> {
    let parent = copy
        .parent()
        .ok_or_else(|| refused(format!("{} has no folder above it", copy.display())))?;
    let name = copy
        .file_name()
        .map_or("database".into(), |n| n.to_string_lossy().into_owned());
    std::fs::create_dir_all(parent).map_err(|e| unreadable(parent, e))?;
    let at = parent.join(format!(".{name}.lock"));
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&at)
    {
        Ok(mut f) => {
            use std::io::Write as _;
            let _ = writeln!(f, "{}", std::process::id());
            Ok(Lock(at))
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Err(Error::new(
            ErrorKind::Io,
            format!(
                "another program is copying the database or writing your copy ({} is there): wait for it, or remove that file if no other program is running",
                at.display()
            ),
        )),
        Err(e) => Err(unreadable(&at, e)),
    }
}

fn refused(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Malformed, message)
}

fn unreadable(at: &Path, e: std::io::Error) -> Error {
    Error::new(ErrorKind::Io, format!("{}: {e}", at.display()))
}

/// A count the meta holds: 1, 2, 3 … As a second writer's revision is told,
/// 0 when the file says none.
fn count(f: &File, key: &str) -> u64 {
    f.meta
        .get(key)
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0)
}

/// The file's revision, 1 or more, or the refusal: a revision that is not a
/// count is never read as none.
pub(crate) fn revision_of(f: &File, path: &str) -> Result<u64, Error> {
    let v = f.meta.get("revision").map_or("", String::as_str);
    match v.trim().parse::<u64>() {
        Ok(n) if n >= 1 => Ok(n),
        _ => Err(refused(format!(
            "{path} says its revision is {v:?}, which is not a count from 1: it is not saved over until it says what it is"
        ))),
    }
}

/// `opened` as the SHA-256 of a file's bytes is written: 64 hex digits. An
/// empty or garbled one is refused as what it is, never told as a writer.
pub fn opened_ok(opened: &str) -> Result<(), Error> {
    if opened.len() == 64
        && opened
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        Ok(())
    } else {
        Err(refused(format!(
            "{opened:?} is not the SHA-256 of the node as it was opened: open it again"
        )))
    }
}

/// The node file at `path`: its kind, its one block, and its place. A node
/// file is `groups/<group_id>/nodes/<block id>.vnode`, and holds one block,
/// the one its `block_uid` names. Its block's id, or the refusal.
pub(crate) fn the_node(f: &File, path: &str) -> Result<String, Error> {
    if f.kind()? != Kind::Node {
        return Err(refused(format!("{path} is not a node file")));
    }
    let uid = f.meta.get("block_uid").map_or("", String::as_str);
    let group_id = f.meta.get("group_id").map_or("", String::as_str);
    let [b] = f.blocks.as_slice() else {
        return Err(refused(format!(
            "{path} holds {} blocks: a node file holds one, its own",
            f.blocks.len()
        )));
    };
    if b.uid != uid {
        return Err(refused(format!(
            "{path} holds the block {:?}, and says it is {uid:?}",
            b.uid
        )));
    }
    let place = format!("groups/{group_id}/nodes/{}.vnode", b.id);
    if path != place {
        return Err(refused(format!(
            "{path} is not where the node it holds is kept: {place}"
        )));
    }
    Ok(b.id.clone())
}

/// What a save never changes: the file but the node's own rows, and but the
/// meta a save writes (the revision, the writer and the program).
fn outside(f: &File, uid: &str) -> File {
    let mut o = f.clone();
    for k in ["revision", "writer", "written_by_app"] {
        o.meta.remove(k);
    }
    o.blocks.retain(|b| b.uid != uid);
    o.ports.retain(|p| p.block_uid != uid);
    o.wires.retain(|w| w.to_block != uid);
    o.closures.retain(|c| c.block_uid != uid);
    o.loops.retain(|l| l.block_uid != uid);
    o.cases.retain(|c| c.block_uid != uid);
    o.texts.retain(|t| t.scope != uid);
    o.tables.retain(|t| t.scope != uid);
    o.media.retain(|m| m.scope != uid);
    o.comments.retain(|c| c.scope != uid);
    o.changes.retain(|c| c.scope != uid);
    o
}

/// The tables of `edit` outside the node `uid` that are not as `now` holds
/// them, by name.
pub(crate) fn changed_outside(edit: &File, now: &File, uid: &str) -> Vec<String> {
    outside(edit, uid)
        .to_tables()
        .into_iter()
        .zip(outside(now, uid).to_tables())
        .filter(|(a, b)| a != b)
        .map(|(a, _)| a.name)
        .collect()
}

/// The second writer: the file as it is now, from the bytes read.
fn second(at: &Path, bytes: Vec<u8>) -> Result<Saved, Error> {
    let now = crate::sqlite::read(at)?;
    Ok(Saved::SecondWriter {
        writer: now.meta.get("writer").cloned().unwrap_or_default(),
        revision: count(&now, "revision"),
        digest: copied::digest(&bytes),
        bytes,
        now: Box::new(now),
    })
}

/// The second writer asking is stopped by, as a save is.
fn second_asked(at: &Path, bytes: Vec<u8>) -> Result<Asked, Error> {
    let now = crate::sqlite::read(at)?;
    Ok(Asked::SecondWriter {
        writer: now.meta.get("writer").cloned().unwrap_or_default(),
        revision: count(&now, "revision"),
        digest: copied::digest(&bytes),
        bytes,
        now: Box::new(now),
    })
}

/// Save `edit` as the next revision of the node at `path` inside the copy
/// at `copy`, written by `by` with the program `app`, when the bytes there are
/// still the ones whose SHA-256 is `opened`. `group` is the node's group file,
/// opened through the chain: it says who is assigned the block.
pub fn save(
    copy: &Path,
    path: &str,
    edit: &File,
    opened: &str,
    by: &str,
    app: &str,
    group: &Registry,
) -> Result<Saved, Error> {
    if !copied::path_ok(path) {
        return Err(refused(format!("{path:?} is not a path inside the copy")));
    }
    opened_ok(opened)?;
    // One writer at a time, from the first read to the rename.
    let _held = lock(copy)?;
    let at = copy.join(path);
    let now = crate::sqlite::read(&at)?;
    let id = the_node(&now, path)?;
    for key in ["file_kind", "group_id", "block_uid"] {
        if edit.meta.get(key) != now.meta.get(key) {
            return Err(refused(format!(
                "the edit says it is {} {:?}, and the file is {:?}: a save does not move a node",
                key,
                edit.meta.get(key).map_or("", String::as_str),
                now.meta.get(key).map_or("", String::as_str)
            )));
        }
    }
    let block = now.blocks[0].clone();
    let group_id = now.meta.get("group_id").cloned().unwrap_or_default();
    if group.file.meta.get("group_id") != Some(&group_id) {
        return Err(refused(format!(
            "the node is {group_id}'s, and the group file given is {}'s",
            group
                .file
                .meta
                .get("group_id")
                .map_or("no group", String::as_str)
        )));
    }
    if !group.may_sign_node(by, &block.uid) {
        return Err(Error::new(
            ErrorKind::Signature,
            format!(
                "{by} is not assigned {id} in {}, nor the deputy of whoever is: one writer per file",
                group.from
            ),
        ));
    }
    if edit.requests != now.requests {
        return Err(refused(format!(
            "the edit changes the requests in {id}'s file: a request is kept as it was asked, and one is added only by asking"
        )));
    }
    let changed = changed_outside(edit, &now, &block.uid);
    if !changed.is_empty() {
        return Err(refused(format!(
            "the edit changes what is not {id}'s own to change ({}): a save changes the node's own rows, and nothing else in its file",
            changed.join(", ")
        )));
    }
    let Some(mine) = edit.blocks.iter().find(|b| b.uid == block.uid) else {
        return Err(refused(format!("the edit does not hold {id}'s block")));
    };
    if (&mine.id, &mine.parent_uid, mine.contract_version)
        != (&block.id, &block.parent_uid, block.contract_version)
    {
        return Err(refused(format!(
            "the edit changes {id}'s id, the block it hangs from or its contract's version: a save does not move a node, and a contract changes by a request"
        )));
    }
    let revision = revision_of(&now, path)?.max(block.revision.max(0) as u64) + 1;
    let mut next = edit.clone();
    next.meta.insert("revision".into(), revision.to_string());
    next.meta.insert("writer".into(), by.to_string());
    next.meta.insert("written_by_app".into(), app.to_string());
    for b in next.blocks.iter_mut().filter(|b| b.uid == block.uid) {
        b.revision = revision as i64;
    }
    next.check_meta()?;
    let findings = checks::check_release(&next);
    // The one comparison, as late as can be before the write: a writer who
    // saved since the person opened it is stopped, and nothing is written.
    let again = std::fs::read(&at).map_err(|e| unreadable(&at, e))?;
    if copied::digest(&again) != opened {
        return second(&at, again);
    }
    let written = crate::sqlite::written(&next, &at)?;
    Ok(Saved::Written {
        revision,
        digest: copied::digest(&written),
        findings,
    })
}

/// Sign the node at `path` inside the copy as `by`, as the day's work
/// (docs/briefs/F3.md, F3-b), when its bytes are still the ones whose SHA-256
/// is `opened`: what the person signs is exactly what they saw, read from
/// those bytes. Refused, with nothing written, by `chain::sign_node`: unless
/// `key` is `by`'s, counted at `at`, and `by` may sign the node; then while
/// any of the signed file's own checks fails, its cases disagree with its
/// method, or it holds nothing to sign, each one named. The revision does not
/// rise: a signature is of the revision as it is. Written as a save is, by
/// `app`, under the copy's lock, and a second writer is stopped the same way.
#[allow(clippy::too_many_arguments)]
pub fn sign_in_place(
    copy: &Path,
    path: &str,
    opened: &str,
    by: &str,
    key: &crate::keys::SigningKey,
    at: &str,
    app: &str,
    group: &Registry,
) -> Result<Saved, Error> {
    if !copied::path_ok(path) {
        return Err(refused(format!("{path:?} is not a path inside the copy")));
    }
    opened_ok(opened)?;
    let _held = lock(copy)?;
    let at_path = copy.join(path);
    let bytes = std::fs::read(&at_path).map_err(|e| unreadable(&at_path, e))?;
    // What changed since it was seen is a second writer before anything
    // else: its checks are not the person's to be told.
    if copied::digest(&bytes) != opened {
        return second(&at_path, bytes);
    }
    // What is signed is read from the bytes compared, not from the path
    // again: what the person saw, and nothing that came between.
    let now = parsed(&bytes)?;
    the_node(&now, path)?;
    let group_id = now.meta.get("group_id").cloned().unwrap_or_default();
    if group.file.meta.get("group_id") != Some(&group_id) {
        return Err(refused(format!(
            "the node is {group_id}'s, and the group file given is {}'s",
            group
                .file
                .meta
                .get("group_id")
                .map_or("no group", String::as_str)
        )));
    }
    // A node file is one revision: the file's and its block's say the same,
    // or what a signature names is not what the file says it is.
    let revision = revision_of(&now, path)?;
    if now.blocks[0].revision != revision as i64 {
        return Err(refused(format!(
            "{path} says it is revision {revision}, and its block says {}: it is one revision, and it is not signed until the two agree",
            now.blocks[0].revision
        )));
    }
    // Who signs first, then what: `sign_node` holds the signed file to the
    // node's checks, and its new signature to the chain.
    let block = now.blocks[0].uid.clone();
    let (mut signed, found) = crate::chain::sign_node(&now, &block, by, key, at, group)?;
    signed.meta.insert("written_by_app".into(), app.to_string());
    // As a save: the one late comparison before the write.
    let again = std::fs::read(&at_path).map_err(|e| unreadable(&at_path, e))?;
    if copied::digest(&again) != opened {
        return second(&at_path, again);
    }
    let written = crate::sqlite::written(&signed, &at_path)?;
    Ok(Saved::Written {
        revision,
        digest: copied::digest(&written),
        findings: found,
    })
}

/// What asking did, or why it did nothing.
#[derive(Debug)]
pub enum Asked {
    /// The request `uid` is written, signed; the file's bytes are now
    /// `digest`.
    Written { uid: String, digest: String },
    /// The file changed in the copy since it was opened: nothing is written.
    /// It is as `now` holds it, last written by `writer` as `revision`.
    SecondWriter {
        now: Box<File>,
        bytes: Vec<u8>,
        writer: String,
        revision: u64,
        digest: String,
    },
}

/// Ask, as `by`, for a contract change or a breakdown of the node at `path`
/// inside the copy (docs/briefs/F3.md, F3-d; W4, W13), when its bytes are
/// still the ones whose SHA-256 is `opened`. The request is written in the
/// node's file and signed on its own by `chain::ask`, which refuses, with
/// nothing written, unless `key` is `by`'s, counted at `at`, and `by` may
/// write the node. The node is not touched: its revision and content, and
/// every signature of it, are as they were, so a node that cannot yet be
/// signed can still ask. Written as a save is, by `app`, under the copy's
/// lock, and a second writer is stopped the same way.
#[allow(clippy::too_many_arguments)]
pub fn ask_in_place(
    copy: &Path,
    path: &str,
    opened: &str,
    by: &str,
    key: &crate::keys::SigningKey,
    at: &str,
    kind: &str,
    body: &str,
    app: &str,
    group: &Registry,
) -> Result<Asked, Error> {
    if !copied::path_ok(path) {
        return Err(refused(format!("{path:?} is not a path inside the copy")));
    }
    opened_ok(opened)?;
    let _held = lock(copy)?;
    let at_path = copy.join(path);
    let bytes = std::fs::read(&at_path).map_err(|e| unreadable(&at_path, e))?;
    if copied::digest(&bytes) != opened {
        return second_asked(&at_path, bytes);
    }
    // What is asked of is read from the bytes compared: what the person saw.
    let now = parsed(&bytes)?;
    the_node(&now, path)?;
    let group_id = now.meta.get("group_id").cloned().unwrap_or_default();
    if group.file.meta.get("group_id") != Some(&group_id) {
        return Err(refused(format!(
            "the node is {group_id}'s, and the group file given is {}'s",
            group
                .file
                .meta
                .get("group_id")
                .map_or("no group", String::as_str)
        )));
    }
    // One revision, as a signing holds it: a request asked of a file whose
    // two revisions disagree could never go back.
    let revision = revision_of(&now, path)?;
    if now.blocks[0].revision != revision as i64 {
        return Err(refused(format!(
            "{path} says it is revision {revision}, and its block says {}: it is one revision, and nothing is asked of it until the two agree",
            now.blocks[0].revision
        )));
    }
    let block = now.blocks[0].uid.clone();
    let (mut asked, uid) = crate::chain::ask(&now, &block, kind, body, by, key, at, group)?;
    asked.meta.insert("written_by_app".into(), app.to_string());
    asked.check_meta()?;
    let again = std::fs::read(&at_path).map_err(|e| unreadable(&at_path, e))?;
    if copied::digest(&again) != opened {
        return second_asked(&at_path, again);
    }
    let written = crate::sqlite::written(&asked, &at_path)?;
    Ok(Asked::Written {
        uid,
        digest: copied::digest(&written),
    })
}

/// The file `bytes` are, read through a private file of its own in the
/// system's temporary folder, made new and gone once read: the bytes alone,
/// with nothing beside them (no journal) that they do not hold.
pub(crate) fn parsed(bytes: &[u8]) -> Result<File, Error> {
    use std::io::Write as _;
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let at = std::env::temp_dir().join(format!(
        "vleo-signing-{}-{nanos}-{}.vnode",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&at)
        .map_err(|e| unreadable(&at, e))?;
    let wrote = f.write_all(bytes).map_err(|e| unreadable(&at, e));
    drop(f);
    let read = wrote.and_then(|_| crate::sqlite::read(&at));
    let _ = std::fs::remove_file(&at);
    read
}
