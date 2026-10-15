//! A node's file, read from the working copy and saved back to it as the
//! next revision (docs/briefs/F3.md, F3-a; `vleo_files::save`).
//!
//! `GET /v1/node/<id>/file` gives the node file's bytes, base64, with the
//! SHA-256 of those bytes (`opened`), its revision and its writer.
//! `POST /v1/node/<id>/save` takes the edited file the same way, with the
//! `opened` it was edited from, and saves it as whoever's key is open. On a
//! second writer it answers both: the file as it now is, and the edit sent,
//! so the screen shows them and the person keeps one. Saving needs the open
//! key and the launch's word, but no signature; the drive is never written.

use super::*;
use vleo_files::save::{save, sign_in_place, Saved};
use vleo_files::sqlite;

/// A node's id as its file is named: letters, digits and `_`.
fn node_id(id: &str) -> Result<&str, String> {
    if id.is_empty() || !id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') {
        return Err(format!("{id:?} is not a node's id"));
    }
    Ok(id)
}

/// Where the node's file is in the copy, as a path inside it: the one
/// `groups/<group>/nodes/<id>.vnode` there is. Two are refused, by name.
fn find(copy: &Path, id: &str) -> Result<String, String> {
    let groups = copy.join("groups");
    let mut found = Vec::new();
    for g in std::fs::read_dir(&groups).map_err(|e| format!("{}: {e}", groups.display()))? {
        let g = g.map_err(|e| format!("{}: {e}", groups.display()))?;
        let rel = format!(
            "groups/{}/nodes/{id}.vnode",
            g.file_name().to_string_lossy()
        );
        if copy.join(&rel).is_file() {
            found.push(rel);
        }
    }
    found.sort();
    match found.len() {
        0 => Err(format!(
            "no node file {id}.vnode in your copy of the database"
        )),
        1 => Ok(found.remove(0)),
        _ => Err(format!(
            "{id} has a file in more than one group, so neither is opened: {}",
            found.join(", ")
        )),
    }
}

/// Standard base64, as `results_file::unbase64` reads it back.
fn base64(bytes: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (c[0] as u32) << 16
            | (*c.get(1).unwrap_or(&0) as u32) << 8
            | *c.get(2).unwrap_or(&0) as u32;
        for k in 0..4 {
            if k <= c.len() {
                out.push(A[(n >> (18 - 6 * k) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// The program, as `written_by_app` names it in what it writes.
const APP: &str = concat!("vleo ", env!("CARGO_PKG_VERSION"));

/// The edit sent, read as a file. It is held in the system's temporary folder
/// under a name of its own, made new, never inside the copy, and gone once
/// read.
fn held(bytes: &[u8]) -> Result<vleo_files::model::File, String> {
    use std::io::Write as _;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let at = std::env::temp_dir().join(format!("vleo-edit-{}-{nanos}.vnode", std::process::id()));
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&at)
        .map_err(|e| format!("the edit could not be held: {e}"))?;
    let wrote = f.write_all(bytes);
    drop(f);
    let edit = wrote
        .map_err(|e| format!("the edit could not be held: {e}"))
        .and_then(|_| {
            sqlite::read(&at).map_err(|e| format!("the edit is not a node file: {}", e.message()))
        });
    let _ = std::fs::remove_file(&at);
    edit
}

/// The copy the program reads, once it is one.
fn the_copy(ctx: &Ctx) -> Result<PathBuf, String> {
    let copy = copy::dir(ctx);
    if copy::record(&copy)?.is_none() {
        return Err(
            "a node is opened from your copy of the database: copy it from the drive first".into(),
        );
    }
    Ok(copy)
}

/// The node file at `path`, read from its bytes: the bytes, and the file they
/// are. Read twice and compared, so what is answered is one file, not a
/// file and the one written over it between.
fn read_once(copy: &Path, path: &str) -> Result<(Vec<u8>, vleo_files::model::File), String> {
    let at = copy.join(path);
    let bytes = std::fs::read(&at).map_err(|e| format!("{path}: {e}"))?;
    let f = sqlite::read(&at).map_err(|e| format!("{path}: {}", e.message()))?;
    if std::fs::read(&at).map_err(|e| format!("{path}: {e}"))? != bytes {
        return Err(format!(
            "{path} was written while it was opened: open it again"
        ));
    }
    Ok((bytes, f))
}

/// `GET /v1/node/<id>/file`.
pub(super) fn file(id: &str, ctx: &Ctx) -> String {
    let answer = || -> Result<String, String> {
        let id = node_id(id)?;
        let copy = the_copy(ctx)?;
        let path = find(&copy, id)?;
        let (bytes, f) = read_once(&copy, &path)?;
        let mut j = Json::new();
        j.raw("{");
        j.bool_field("ok", true);
        j.str_field("node", id);
        j.str_field("path", &path);
        j.str_field("opened", &vleo_files::copied::digest(&bytes));
        j.str_field(
            "revision",
            f.meta.get("revision").map_or("", String::as_str),
        );
        j.str_field("writer", f.meta.get("writer").map_or("", String::as_str));
        j.str_field("file", &base64(&bytes));
        j.raw("}");
        Ok(j.0)
    };
    answer().unwrap_or_else(|why| me::refusal(&why))
}

/// The group file of the node at `path`, as the chain opens it from the copy.
/// The group is the folder the file is in: the library holds the file to it,
/// so an edit that names another group is refused there, by name.
fn group_of(copy: &Path, path: &str) -> Result<vleo_files::chain::Registry, String> {
    let c =
        match me::registries(copy, copy, "") {
            Ok(c) => c,
            Err(me::NotRead::FirstUse(..)) => return Err(
                "this computer has not accepted this database's anchor: open your key on it first"
                    .into(),
            ),
            Err(me::NotRead::Why(why)) => return Err(why),
        };
    let group_id = path.split('/').nth(1).unwrap_or_default().to_string();
    let why = c
        .left_out
        .iter()
        .find(|w| w.starts_with(&format!("{group_id}: ")))
        .map_or(String::new(), |w| format!(": {w}"));
    c.groups
        .into_iter()
        .find(|g| g.file.meta.get("group_id") == Some(&group_id))
        .ok_or_else(|| {
            format!("{group_id}'s group file does not check through the chain, so nobody is assigned its nodes{why}")
        })
}

/// `findings`, each with its level, place and what it says.
fn findings_field(j: &mut Json, findings: &vleo_files::checks::Findings) {
    j.key("findings").raw("[");
    for (i, f) in findings.0.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field(
            "level",
            match f.level {
                vleo_files::checks::Level::Error => "error",
                vleo_files::checks::Level::Warning => "warning",
            },
        );
        j.str_field("place", &f.place);
        j.str_field("what", &f.what);
        j.raw("}");
    }
    j.raw("]");
}

/// `POST /v1/node/<id>/save`.
pub(super) fn save_route(id: &str, params: &str, ctx: &Ctx) -> String {
    match saved(id, params, ctx) {
        Ok(j) => j,
        Err(why) => me::refusal(&why),
    }
}

fn saved(id: &str, params: &str, ctx: &Ctx) -> Result<String, String> {
    me::carries_session(params)?;
    // Who writes it is the first thing a save needs: without a key there is
    // nobody to write it as, whatever else is wrong.
    if me::with_key(|_, _| ()).is_none() {
        return Err("no key is open: open yours, and the revision is written as you".into());
    }
    let id = node_id(id)?;
    let copy = the_copy(ctx)?;
    let opened = param(params, "opened").map(decode).unwrap_or_default();
    let sent = param(params, "file").map(decode).unwrap_or_default();
    let bytes = results_file::unbase64(&sent)
        .ok_or("the edit is not a node file, as base64: nothing is saved")?;
    let path = find(&copy, id)?;
    let edit = held(&bytes)?;
    let group = group_of(&copy, &path)?;
    let group = &group;
    let done = me::with_key(|key, me| {
        // The key is the one the group file registers for them, now.
        group
            .key(&me.name, &key.public().to_hex(), &results::now_utc())
            .map_err(|e| e.message().to_string())?;
        save(&copy, &path, &edit, &opened, &me.name, APP, group)
            .map(|s| (s, me.name.clone()))
            .map_err(|e| e.message().to_string())
    })
    .ok_or("no key is open: open yours, and the revision is written as you")??;
    let mut j = Json::new();
    j.raw("{");
    match done {
        (
            Saved::Written {
                revision,
                digest,
                findings,
            },
            by,
        ) => {
            j.bool_field("ok", true);
            j.str_field("node", id);
            j.str_field("path", &path);
            j.num_field("revision", revision as f64);
            j.str_field("opened", &digest);
            findings_field(&mut j, &findings);
            j.str_field(
                "message",
                &format!(
                    "{id} is saved as revision {revision}, written by {by}, in your copy. Not signed: what its content still lacks is listed."
                ),
            );
        }
        (
            Saved::SecondWriter {
                bytes: theirs,
                writer,
                revision,
                digest,
                ..
            },
            _,
        ) => {
            j.bool_field("ok", false);
            j.bool_field("conflict", true);
            j.str_field("node", id);
            j.str_field(
                "message",
                &format!(
                    "{id} changed in your copy since you opened it: {writer} saved revision {revision}. Nothing is written; both are here, and you keep one."
                ),
            );
            j.str_field("writer", &writer);
            j.num_field("revision", revision as f64);
            j.str_field("opened", &digest);
            j.str_field("theirs", &base64(&theirs));
            j.str_field("yours", &sent);
        }
    }
    j.raw("}");
    Ok(j.0)
}

/// `POST /v1/node/<id>/sign`: the node signed as the day's work by whoever's
/// key is open, at this moment, when its file is still the one opened
/// (docs/briefs/F3.md, F3-b). Refused while any of its checks fails or its
/// cases disagree with its method, each one named; refused unless the open
/// key is the one its group file registers for them, and they are assigned
/// the node or are the deputy of whoever is. The signed file is written in
/// the copy as a save is; the drive is never written.
pub(super) fn sign_route(id: &str, params: &str, ctx: &Ctx) -> String {
    match signed(id, params, ctx) {
        Ok(j) => j,
        Err(why) => me::refusal(&why),
    }
}

fn signed(id: &str, params: &str, ctx: &Ctx) -> Result<String, String> {
    me::carries_session(params)?;
    if me::with_key(|_, _| ()).is_none() {
        return Err("no key is open: open yours, and the node is signed with it".into());
    }
    let id = node_id(id)?;
    let copy = the_copy(ctx)?;
    let opened = param(params, "opened").map(decode).unwrap_or_default();
    let path = find(&copy, id)?;
    let group = group_of(&copy, &path)?;
    let at = results::now_utc();
    // A group file changed on the drive since the copy may revoke what the
    // copy's still counts: said beside the signing, and checked again when
    // the work is put back (F3-c).
    let newer = me::newer_on_drive(&copy, &copy)?;
    let done = me::with_key(|key, me| {
        sign_in_place(&copy, &path, &opened, &me.name, key, &at, APP, &group)
            .map(|s| (s, me.name.clone()))
            .map_err(|e| e.message().to_string())
    })
    .ok_or("no key is open: open yours, and the node is signed with it")??;
    let mut j = Json::new();
    j.raw("{");
    match done {
        (
            Saved::Written {
                revision,
                digest,
                findings,
            },
            by,
        ) => {
            j.bool_field("ok", true);
            j.str_field("node", id);
            j.str_field("path", &path);
            j.num_field("revision", revision as f64);
            j.str_field("opened", &digest);
            j.str_field("signed_by", &by);
            j.str_field("signed_at", &at);
            findings_field(&mut j, &findings);
            j.key("newer_on_drive").raw("[");
            for (i, f) in newer.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                j.raw(&json::string(f));
            }
            j.raw("]");
            j.str_field(
                "message",
                &format!(
                    "{id} is signed by {by} as revision {revision}, in your copy. It goes to the drive when your work is put back."
                ),
            );
        }
        (
            Saved::SecondWriter {
                bytes: theirs,
                writer,
                revision,
                digest,
                ..
            },
            _,
        ) => {
            j.bool_field("ok", false);
            j.bool_field("conflict", true);
            j.str_field("node", id);
            j.str_field(
                "message",
                &format!(
                    "{id} changed in your copy since you opened it: {writer} saved revision {revision}. Nothing is signed: open it again, and sign what you see."
                ),
            );
            j.str_field("writer", &writer);
            j.num_field("revision", revision as f64);
            j.str_field("opened", &digest);
            j.str_field("theirs", &base64(&theirs));
        }
    }
    j.raw("}");
    Ok(j.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_reads_back_as_it_was_written() {
        for n in 0..10usize {
            let bytes: Vec<u8> = (0..n as u8).map(|b| b.wrapping_mul(37)).collect();
            assert_eq!(results_file::unbase64(&base64(&bytes)), Some(bytes));
        }
    }
}
