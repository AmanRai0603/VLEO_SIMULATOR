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
use vleo_files::save::{save, Saved};
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
    let mut found: Vec<String> = std::fs::read_dir(copy.join("groups"))
        .map_err(|e| format!("{}: {e}", copy.join("groups").display()))?
        .flatten()
        .filter_map(|g| {
            let group = g.file_name().to_string_lossy().into_owned();
            let rel = format!("groups/{group}/nodes/{id}.vnode");
            copy.join(&rel).is_file().then_some(rel)
        })
        .collect();
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

/// `GET /v1/node/<id>/file`.
pub(super) fn file(id: &str, ctx: &Ctx) -> String {
    let answer = || -> Result<String, String> {
        let id = node_id(id)?;
        let copy = the_copy(ctx)?;
        let path = find(&copy, id)?;
        let bytes = std::fs::read(copy.join(&path)).map_err(|e| format!("{path}: {e}"))?;
        let f = sqlite::read(&copy.join(&path)).map_err(|e| format!("{path}: {}", e.message()))?;
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
    // One program at a time writes the copy, and none copies over it meanwhile.
    let _held = copy::lock(&copy)?;
    let path = find(&copy, id)?;
    // The edit is read as a file beside the copy's own, never inside it.
    let held = copy.with_file_name(format!(
        ".{}.edit-{}",
        copy.file_name()
            .map_or("copy".into(), |n| n.to_string_lossy()),
        std::process::id()
    ));
    std::fs::write(&held, &bytes).map_err(|e| format!("the edit could not be held: {e}"))?;
    let edit = sqlite::read(&held);
    let _ = std::fs::remove_file(&held);
    let edit = edit.map_err(|e| format!("the edit is not a node file: {}", e.message()))?;
    let c =
        match me::registries(&copy, &copy, "") {
            Ok(c) => c,
            Err(me::NotRead::FirstUse(..)) => return Err(
                "this computer has not accepted this database's anchor: open your key on it first"
                    .into(),
            ),
            Err(me::NotRead::Why(why)) => return Err(why),
        };
    let group_id = edit.meta.get("group_id").cloned().unwrap_or_default();
    let group = c
        .groups
        .iter()
        .find(|g| g.file.meta.get("group_id") == Some(&group_id))
        .ok_or_else(|| {
            format!("{group_id}'s group file does not check through the chain, so nobody is assigned its nodes")
        })?;
    let done = me::with_key(|_, me| {
        save(&copy, &path, &edit, &opened, &me.name, group)
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
            j.str_field(
                "message",
                &format!(
                    "{id} is saved as revision {revision}, written by {by}, in your copy. Not signed: what its content still lacks is listed."
                ),
            );
        }
        (
            Saved::SecondWriter {
                now,
                writer,
                revision,
                digest,
            },
            _,
        ) => {
            let theirs = std::fs::read(copy.join(&path)).map_err(|e| format!("{path}: {e}"))?;
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
            let _ = now;
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
