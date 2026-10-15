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
//!   block, or they are the deputy of whoever it does. The edit must also be
//!   the same node: its group and its block cannot change in a save.
//!
//! Otherwise the file is checked as a file (its meta and its values, as every
//! write checks them), its revision rises by one and its writer is the
//! person, and it is written whole: to a file beside it, then renamed over
//! the old one (`sqlite::write`). What its content still lacks is said beside
//! the save, never refused here: a day's work is saved while it is still
//! work. Signing it (F3-b) is where the content must hold.
//!
//! The drive is never written here: putting work back is F3-c.

use std::path::Path;

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
        writer: String,
        revision: u64,
        digest: String,
    },
}

fn refused(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Malformed, message)
}

fn unreadable(at: &Path, e: std::io::Error) -> Error {
    Error::new(ErrorKind::Io, format!("{}: {e}", at.display()))
}

/// A count the meta holds: 1, 2, 3 …
fn count(f: &File, key: &str) -> u64 {
    f.meta
        .get(key)
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0)
}

/// Save `edit` as the next revision of the node at `path` inside the copy
/// at `copy`, written by `by`, when the bytes there are still the ones whose
/// SHA-256 is `opened`. `group` is the node's group file, opened through the
/// chain: it says who is assigned the block.
pub fn save(
    copy: &Path,
    path: &str,
    edit: &File,
    opened: &str,
    by: &str,
    group: &Registry,
) -> Result<Saved, Error> {
    if !copied::path_ok(path) {
        return Err(refused(format!("{path:?} is not a path inside the copy")));
    }
    let at = copy.join(path);
    let now = crate::sqlite::read(&at)?;
    if now.kind()? != Kind::Node {
        return Err(refused(format!("{path} is not a node file")));
    }
    for key in ["group_id", "block_uid"] {
        if edit.meta.get(key) != now.meta.get(key) {
            return Err(refused(format!(
                "the edit says it is {} {:?}, and the file is {:?}: a save does not move a node",
                key,
                edit.meta.get(key).map_or("", String::as_str),
                now.meta.get(key).map_or("", String::as_str)
            )));
        }
    }
    let block = now.meta.get("block_uid").cloned().unwrap_or_default();
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
    let assigned = group
        .file
        .assignments
        .iter()
        .any(|a| a.block_uid == block && group.acts_for(by, &a.person));
    if !assigned {
        return Err(Error::new(
            ErrorKind::Signature,
            format!(
                "{by} is not assigned {} in {}, nor the deputy of whoever is: one writer per file",
                now.blocks
                    .iter()
                    .find(|b| b.uid == block)
                    .map_or(block.as_str(), |b| b.id.as_str()),
                group.from
            ),
        ));
    }
    let revision = count(&now, "revision") + 1;
    let mut next = edit.clone();
    next.meta.insert("revision".into(), revision.to_string());
    next.meta.insert("writer".into(), by.to_string());
    for b in next.blocks.iter_mut().filter(|b| b.uid == block) {
        b.revision = revision as i64;
    }
    next.check_meta()?;
    let findings = checks::check_release(&next);
    // The one comparison, as late as can be before the write: a writer who
    // saved since the person opened it, or while this was being checked, is
    // stopped, and nothing is written.
    let again = std::fs::read(&at).map_err(|e| unreadable(&at, e))?;
    if copied::digest(&again) != opened {
        let now = crate::sqlite::read(&at)?;
        return Ok(Saved::SecondWriter {
            writer: now.meta.get("writer").cloned().unwrap_or_default(),
            revision: count(&now, "revision"),
            digest: copied::digest(&again),
            now: Box::new(now),
        });
    }
    crate::sqlite::write(&next, &at)?;
    let written = std::fs::read(&at).map_err(|e| unreadable(&at, e))?;
    Ok(Saved::Written {
        revision,
        digest: copied::digest(&written),
        findings,
    })
}
