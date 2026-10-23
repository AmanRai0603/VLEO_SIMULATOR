//! A node's file, read from the working copy and saved back to it as the
//! next revision (docs/briefs/F3.md, F3-a; `vleo_files::save`).
//!
//! `GET /v1/node/<id>/file` gives the node file's bytes, base64, with the
//! SHA-256 of those bytes (`opened`), its revision and its writer, and what
//! the Node workspace shows of it, which a page cannot read from the bytes:
//! who it is assigned to, whether it is signed as it is now, each signature
//! and request, what its checks find, and the next step (F3-g1).
//! `POST /v1/node/<id>/save` takes the edited file the same way, with the
//! `opened` it was edited from, and saves it as whoever's key is open. On a
//! second writer it answers both: the file as it now is, and the edit sent,
//! so the screen shows them and the person keeps one. Saving needs the open
//! key and the launch's word, but no signature; the drive is never written.
//!
//! `POST /v1/node/<id>/sign` signs it as the day's work (F3-b),
//! `POST /v1/node/<id>/ask` writes a request in it, for a contract change or
//! a breakdown, signed by whoever asks (F3-d), and
//! `POST /v1/node/<id>/lesson` saves the row's lesson in it (F3-e).

use super::*;
use vleo_files::edit::{self, Declared, Edit, Edits, Part, Tried};
use vleo_files::save::{ask_in_place, save, sign_in_place, Asked, Saved};
use vleo_files::sqlite;

/// A node's id as its file is named: letters, digits and `_`.
pub(super) fn node_id(id: &str) -> Result<&str, String> {
    if id.is_empty() || !id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') {
        return Err(format!("{id:?} is not a node's id"));
    }
    Ok(id)
}

/// Where the node's file is in the copy, as a path inside it: the one
/// `groups/<group>/nodes/<id>.vnode` there is. Two are refused, by name.
pub(super) fn find(copy: &Path, id: &str) -> Result<String, String> {
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
pub(super) fn the_copy(ctx: &Ctx) -> Result<PathBuf, String> {
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
pub(super) fn read_once(
    copy: &Path,
    path: &str,
) -> Result<(Vec<u8>, vleo_files::model::File), String> {
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
        state_fields(&mut j, &f, &bytes, &path, &copy, ctx);
        j.raw("}");
        Ok(j.0)
    };
    answer().unwrap_or_else(|why| me::refusal(&why))
}

/// What the Node workspace shows of a node's file beside its bytes, as the
/// program reads it (docs/briefs/F3.md, F3-g1): its group and who the group
/// file assigns it, whether it is the open key's person's to write, whether
/// it is signed as it is now, each signature and request, what its own checks
/// find, and the one next step. A page cannot read the file's bytes, so
/// everything it shows of the file is said here.
fn state_fields(
    j: &mut Json,
    f: &vleo_files::model::File,
    bytes: &[u8],
    path: &str,
    copy: &Path,
    ctx: &Ctx,
) {
    let block = f.meta.get("block_uid").cloned().unwrap_or_default();
    let group_id = path.split('/').nth(1).unwrap_or_default().to_string();
    j.str_field("group", &group_id);
    j.str_field("block", &block);
    let group = group_of(copy, path);
    // Everyone the group file assigns it to: a block may have more than one
    // writer, and an empty name is nobody.
    let assignees: Vec<String> = group.as_ref().map_or_else(
        |_| Vec::new(),
        |g| {
            g.file
                .assignments
                .iter()
                .filter(|a| a.block_uid == block && !a.person.is_empty())
                .map(|a| a.person.clone())
                .collect()
        },
    );
    j.str_field("assigned", &assignees.join(", "));
    // Theirs to write as the library holds it: `Registry::may_sign_node`, the
    // one rule of who may write and sign a node, with a key the group file
    // counts for them now. The program decides again at each step; this says
    // which steps to offer, and why not.
    let now = results::now_utc();
    let me = me::with_key(|key, me| (me.clone(), key.public().to_hex()));
    let mut key_refused = None;
    let yours = match (&me, &group) {
        (Some((me, public)), Ok(g)) if g.may_sign_node(&me.name, &block) => {
            match g.key(&me.name, public, &now) {
                Ok(_) => true,
                Err(e) => {
                    key_refused = Some(e.message().to_string());
                    false
                }
            }
        }
        _ => false,
    };
    j.bool_field("yours", yours);
    // Whose deputy they write it as, when no assignment names them.
    let deputy_for = match (&me, &group) {
        (Some((me, _)), Ok(g)) if yours && !assignees.contains(&me.name) => g
            .file
            .people
            .iter()
            .find(|p| p.name == me.name && assignees.contains(&p.deputy_for))
            .map(|p| p.deputy_for.clone()),
        _ => None,
    };
    j.str_field("deputy_for", deputy_for.as_deref().unwrap_or(""));
    let signed = match &group {
        Ok(g) => vleo_files::chain::check_node(f, &block, g).map_err(|e| e.message().to_string()),
        Err(why) => Err(why.clone()),
    };
    j.key("signed").raw("{");
    j.bool_field("ok", signed.is_ok());
    j.str_field(
        "says",
        &match &signed {
            Ok(()) => format!(
                "revision {} is signed as it is now",
                f.meta.get("revision").map_or("", String::as_str)
            ),
            Err(why) => why.clone(),
        },
    );
    j.raw("}");
    j.key("signatures").raw("[");
    for (i, s) in f.signatures.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("scope", &s.scope);
        j.str_field("revision", &s.revision);
        j.str_field("signer", &s.signer);
        j.str_field("signed_at", &s.signed_at);
        j.str_field("verdict", &s.verdict);
        j.raw("}");
    }
    j.raw("]");
    j.key("requests").raw("[");
    for (i, r) in f.requests.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        let counts = match &group {
            Ok(g) => {
                vleo_files::chain::check_request(f, &r.uid, g).map_err(|e| e.message().to_string())
            }
            Err(why) => Err(why.clone()),
        };
        j.raw("{");
        j.str_field("uid", &r.uid);
        j.str_field("kind", &r.kind);
        j.str_field("status", &r.status);
        j.str_field("raised_by", &r.raised_by);
        j.str_field("at", &r.at);
        j.str_field("body", &r.body);
        j.bool_field("counts", counts.is_ok());
        j.str_field("why", counts.err().as_deref().unwrap_or(""));
        j.raw("}");
    }
    j.raw("]");
    // What a signing is held to, said before anyone signs: the node's own
    // checks, and its lesson against the design this program reads.
    let mut findings = vleo_files::checks::check_node(f);
    let id = f
        .blocks
        .iter()
        .find(|b| b.uid == block)
        .map_or(block.clone(), |b| b.id.clone());
    if let Err(why) = lesson_holds(ctx, &id, f) {
        findings.0.push(vleo_files::checks::Finding {
            level: vleo_files::checks::Level::Error,
            place: id.clone(),
            what: why,
        });
    }
    findings_field(j, &findings);
    let here = vleo_files::copied::digest(bytes);
    let changed_here = copy::record(copy)
        .ok()
        .flatten()
        .is_some_and(|r| r.files.get(path) != Some(&here));
    j.bool_field("changed_here", changed_here);
    let revision = f.meta.get("revision").map_or("", String::as_str);
    let first_error = findings.errors().next();
    // Signing tried without writing anything, as the library would sign it
    // now: what the checks above cannot see (the key, the block, the file's
    // revision against its block's) refuses here in the library's words.
    let would_sign = match (&group, yours, first_error, &signed) {
        (Ok(g), true, None, Err(_)) => me::with_key(|key, me| {
            vleo_files::chain::sign_node(f, &block, &me.name, key, &now, g)
                .map(|_| ())
                .map_err(|e| e.message().to_string())
        })
        .unwrap_or(Ok(())),
        _ => Ok(()),
    };
    let next = match (&group, me.is_some(), yours, first_error, &signed) {
        (Err(why), ..) => format!("Nothing yet: {why}"),
        (_, false, ..) => {
            "Open your key: whether it is yours to write is read from its group file".to_string()
        }
        (_, _, false, ..) if key_refused.is_some() => format!(
            "Nothing: your key does not count for {id} here: {}",
            key_refused.as_deref().unwrap_or("")
        ),
        (_, _, false, ..) => format!(
            "Nothing: {id} is {}'s to write. Read it here; ask them, or raise an issue",
            if assignees.is_empty() {
                "nobody".to_string()
            } else {
                assignees.join(" and ")
            }
        ),
        (_, _, _, Some(e), Err(_)) => format!(
            "Fix what its checks find, then sign: {}: {}",
            e.place, e.what
        ),
        (_, _, _, None, Err(_)) => match &would_sign {
            Ok(()) => format!("Sign revision {revision}"),
            Err(why) => format!("Nothing yet: signing it is refused: {why}"),
        },
        (_, _, _, _, Ok(())) if changed_here => {
            "Put your work back on the drive, from My work".to_string()
        }
        (_, _, _, _, Ok(())) => format!(
            "Nothing: revision {revision} is signed, as it was when your copy was taken from the drive"
        ),
    };
    j.str_field("next", &next);
    content_fields(j, f, &block, yours);
}

/// What the node says, as the Node workspace writes it (F3-g2a): its shape,
/// whether it can be written here and why not, and the four parts with the
/// declaration and the record, as the library reads them from the file.
fn content_fields(j: &mut Json, f: &vleo_files::model::File, block: &str, yours: bool) {
    let shape = edit::shape(f, block);
    j.str_field(
        "shape",
        match shape {
            edit::Shape::Release => "release",
            edit::Shape::Design => "design",
            edit::Shape::Neither => "neither",
        },
    );
    j.bool_field("editable", yours && shape == edit::Shape::Release);
    j.str_field(
        "edit_why_not",
        match (yours, shape) {
            (false, _) => "it is not yours to write: its writer, or their deputy, writes it",
            (true, edit::Shape::Release) => "",
            (true, edit::Shape::Design) => "it is in design/'s shape (its sheet): it is written here once one node shape is picked (F3-g0, question 9); until then it is read and asked of",
            (true, edit::Shape::Neither) => "it holds neither a release's method and cases nor a sheet: it is written here once one node shape is picked (F3-g0, question 9)",
        },
    );
    let text = |kind: &str| {
        f.texts
            .iter()
            .find(|t| t.scope == block && t.kind == kind)
            .map_or(String::new(), |t| t.body.clone())
    };
    let table = |path: &str| {
        f.tables
            .iter()
            .find(|t| t.scope == block && t.path == path)
            .map_or(String::new(), |t| t.csv.clone())
    };
    let d = edit::declared_of(f, block);
    j.key("content").raw("{");
    j.str_field(
        "question",
        f.blocks
            .iter()
            .find(|b| b.uid == block)
            .map_or("", |b| b.question.as_str()),
    );
    j.key("explain").raw("{");
    j.str_field("simply", &edit::section(f, block, Part::Simply));
    j.str_field("breaks", &edit::section(f, block, Part::Breaks));
    j.str_field("wrong", &edit::section(f, block, Part::Wrong));
    j.raw("}");
    j.str_field("method_text", &text("pseudocode"));
    j.str_field("case_csv", &table("results/isolation.csv"));
    j.key("declared").raw("{");
    j.str_field("author", &d.author);
    j.str_field("ai", &d.ai);
    j.str_field("source", &d.source);
    j.str_field("checked_by", &d.checked_by);
    j.raw("}");
    j.str_field("versions", &table("versions.csv"));
    j.raw("}");
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
    c.systems
        .into_iter()
        .chain(c.groups)
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
    lesson_holds(ctx, id, &edit).map_err(|why| format!("{why}. Nothing is saved."))?;
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
    let (_, now) = read_once(&copy, &path)?;
    lesson_holds(ctx, id, &now).map_err(|why| format!("{why}. Nothing is signed."))?;
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

/// `POST /v1/node/<id>/ask`: a request for a contract change (W4) or a
/// breakdown (W13), `kind` `contract` or `breakdown`, saying `body`, written
/// in the node's file and signed by whoever's key is open, at this moment,
/// when its file is still the one opened (docs/briefs/F3.md, F3-d). Refused
/// unless the open key is the one its group file registers for them, and
/// they are assigned the node or are the deputy of whoever is. The node is
/// not touched, so it asks whether or not it can be signed yet. Written in
/// the copy as a save is; the drive is never written.
pub(super) fn ask_route(id: &str, params: &str, ctx: &Ctx) -> String {
    match asked(id, params, ctx) {
        Ok(j) => j,
        Err(why) => me::refusal(&why),
    }
}

fn asked(id: &str, params: &str, ctx: &Ctx) -> Result<String, String> {
    me::carries_session(params)?;
    if me::with_key(|_, _| ()).is_none() {
        return Err("no key is open: open yours, and the request is signed with it".into());
    }
    let id = node_id(id)?;
    let copy = the_copy(ctx)?;
    let opened = param(params, "opened").map(decode).unwrap_or_default();
    let kind = param(params, "kind").map(decode).unwrap_or_default();
    let body = param(params, "body").map(decode).unwrap_or_default();
    let path = find(&copy, id)?;
    let group = group_of(&copy, &path)?;
    let at = results::now_utc();
    let done = me::with_key(|key, me| {
        ask_in_place(
            &copy, &path, &opened, &me.name, key, &at, &kind, &body, APP, &group,
        )
        .map(|s| (s, me.name.clone()))
        .map_err(|e| e.message().to_string())
    })
    .ok_or("no key is open: open yours, and the request is signed with it")??;
    let mut j = Json::new();
    j.raw("{");
    match done {
        (Asked::Written { uid, digest }, by) => {
            j.bool_field("ok", true);
            j.str_field("node", id);
            j.str_field("path", &path);
            j.str_field("request", &uid);
            j.str_field("kind", &kind);
            j.str_field("opened", &digest);
            j.str_field("asked_by", &by);
            j.str_field("asked_at", &at);
            j.str_field(
                "message",
                &format!(
                    "{by} asks, in {id}'s file, for {}, signed, in your copy. It goes back to the drive when your work is put back: by itself while {id} is the drive's revision, or with a later revision once you sign it. {id}'s subsystem engineer finds it in My work when they copy the drive again.",
                    if kind == "breakdown" {
                        "a breakdown"
                    } else {
                        "a contract change"
                    }
                ),
            );
        }
        (
            Asked::SecondWriter {
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
                    "{id} changed in your copy since you opened it: {writer} saved revision {revision}. Nothing is asked: open it again, and ask of what you see."
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

/// The row's lesson in `f`, if it holds one, held to all the gate checks of
/// it against the design this program reads: what it names there too, which
/// a node's own checks cannot see (`checks::lesson`). So a lesson naming a
/// row that is not there is never saved or signed, by whichever route.
fn lesson_holds(ctx: &Ctx, id: &str, f: &vleo_files::model::File) -> Result<(), String> {
    let uid = f.meta.get("block_uid").map_or("", String::as_str);
    let Some(t) = f
        .texts
        .iter()
        .find(|t| t.scope == uid && t.kind == vleo_files::convert::LESSON)
    else {
        return Ok(());
    };
    let lesson = vleo_sheet::lesson::read(&t.body, id).map_err(|e| {
        format!(
            "{id}'s lesson does not read: {}",
            e.message().trim_end().trim_end_matches('.')
        )
    })?;
    let tree = ctx.load().map_err(|e| {
        format!("the design does not load, so {id}'s lesson cannot be checked against it: {e}")
    })?;
    let bad = vleo_sheet::lesson::problems(&lesson, &tree);
    if bad.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{id}'s lesson does not pass its check: {}",
            bad.join("; ")
        ))
    }
}

/// `POST /v1/node/<id>/lesson`: the row's lesson (docs/LESSONS.md), from a
/// filled lesson form or a bare `lesson.toml` (`lesson`), saved in the node's
/// file in the copy as its next revision, when its file is still the one
/// opened (docs/briefs/F3.md, F3-e). Checked first as the gate checks it,
/// against the design this program reads, and refused with every reason, with
/// nothing written. Saved as any save is: by whoever's key is open, only when
/// they are assigned the node or are the deputy of whoever is. Not signed: the
/// lesson is signed with the node, as the day's work.
pub(super) fn lesson_route(id: &str, params: &str, ctx: &Ctx) -> String {
    match lesson_saved(id, params, ctx) {
        Ok(j) => j,
        Err(why) => me::refusal(&why),
    }
}

fn lesson_saved(id: &str, params: &str, ctx: &Ctx) -> Result<String, String> {
    me::carries_session(params)?;
    if me::with_key(|_, _| ()).is_none() {
        return Err("no key is open: open yours, and the lesson is saved as you".into());
    }
    let id = node_id(id)?;
    let copy = the_copy(ctx)?;
    let opened = param(params, "opened").map(decode).unwrap_or_default();
    let sent = param(params, "lesson").map(decode).unwrap_or_default();
    // A form made for another row is refused by the reader, by name.
    let (_, text) = vleo_sheet::lesson_form::from_file(&sent, Some(id)).map_err(|e| {
        format!(
            "{}. Nothing is saved.",
            e.message().trim_end().trim_end_matches('.')
        )
    })?;
    let path = find(&copy, id)?;
    let (_, now) = read_once(&copy, &path)?;
    let uid = now
        .meta
        .get("block_uid")
        .cloned()
        .ok_or_else(|| format!("{path} names no block"))?;
    let mut edit = now.clone();
    edit.texts
        .retain(|t| !(t.scope == uid && t.kind == vleo_files::convert::LESSON));
    let title = vleo_sheet::lesson::read(&text, id)
        .map(|l| l.title)
        .unwrap_or_default();
    edit.texts.push(vleo_files::model::Text {
        scope: uid,
        kind: vleo_files::convert::LESSON.to_string(),
        body: text,
    });
    lesson_holds(ctx, id, &edit).map_err(|why| format!("{why}. Nothing is saved."))?;
    let group = group_of(&copy, &path)?;
    let group = &group;
    let done = me::with_key(|key, me| {
        group
            .key(&me.name, &key.public().to_hex(), &results::now_utc())
            .map_err(|e| e.message().to_string())?;
        save(&copy, &path, &edit, &opened, &me.name, APP, group)
            .map(|s| (s, me.name.clone()))
            .map_err(|e| e.message().to_string())
    })
    .ok_or("no key is open: open yours, and the lesson is saved as you")??;
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
            j.str_field("title", &title);
            findings_field(&mut j, &findings);
            j.str_field(
                "message",
                &format!(
                    "{id}'s lesson is saved as revision {revision}, written by {by}, in your copy. Not signed: it is signed with the node, as the day's work."
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
                    "{id} changed in your copy since you opened it: {writer} saved revision {revision}. Nothing is saved: open it again, and save the lesson to what you see."
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

/// `POST /v1/node/<id>/edit`: the node written by plain edits as its next
/// revision in the copy (docs/briefs/F3.md, F3-g2a), or, with `dry_run`, the
/// same edits checked as they are typed, with nothing written. Each part is
/// sent only when it changes: `question`, `explain_simply`, `explain_breaks`,
/// `explain_wrong`, `method_text`, `case_csv`, or `formula` in place of
/// `method_text`, with `formula.units` and `formula.answer` for an empirical
/// fit (F8-a3, read as `formula_of` says), or `case_paste` in place of
/// `case_csv`, a table pasted or a CSV file's text, with
/// `case_paste.origin` unless the table has its own (F8-b1); with a method or cases,
/// `declared.author`, `.ai`, `.source` and `.checked_by`; with a decision,
/// the record `why.believed`, `.tested`, `.learned`, `.changed`, `.risks`,
/// `.cost`, `.rests_on` and `.breaks_if`. Every rule is the library's
/// (`vleo_files::edit`); a lesson the edit would leave pointing at nothing is
/// refused, as a save's is. Carries the launch's word; the drive is never
/// written.
pub(super) fn edit_route(id: &str, params: &str, ctx: &Ctx) -> String {
    match edited(id, params, ctx) {
        Ok(j) => j,
        Err(why) => me::refusal(&why),
    }
}

/// A formula as sent (F8-a3): `formula`, and for an empirical fit
/// `formula.units`, one `<name>=<unit>` a line for each input the formula
/// reads, and `formula.answer`, the unit its answer is given in. A line that
/// is not one is kept whole as a name with no unit, which the reader refuses
/// by name.
fn formula_of(params: &str, typed: String) -> edit::Formula {
    let given = |k: &str| param(params, k).map(decode).unwrap_or_default();
    let (units, answer) = (given("formula.units"), given("formula.answer"));
    let fit = (!units.trim().is_empty() || !answer.trim().is_empty()).then(|| {
        vleo_sheet::method::formula::Fit {
            inputs: units
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(|l| match l.split_once('=') {
                    Some((n, u)) => (n.trim().to_string(), u.trim().to_string()),
                    None => (l.to_string(), String::new()),
                })
                .collect(),
            answer: answer.trim().to_string(),
        }
    });
    edit::Formula { typed, fit }
}

/// What a table pasted was read as (F8-b1), beside what the dry run found:
/// the cases it becomes, how many, and each number cell as it was typed and
/// what it was read as, or why the table does not read.
fn table_field(j: &mut Json, file: &vleo_files::model::File, e: &Edits) {
    let Some(t) = e.edits.iter().find_map(|x| match x {
        Edit::Table(t) => Some(t),
        _ => None,
    }) else {
        return;
    };
    let Some(uid) = file.blocks.first().map(|b| b.uid.clone()) else {
        return;
    };
    j.key("table").raw("{");
    match edit::table_of(file, &uid, t) {
        Err(why) => {
            j.bool_field("ok", false);
            j.str_field("refused", why.message());
            j.key("csv").raw("null");
            j.num_field("cases", 0.0);
            j.key("cells").raw("[]");
        }
        Ok(r) => {
            j.bool_field("ok", true);
            j.key("refused").raw("null");
            j.str_field("csv", &r.csv);
            j.num_field("cases", r.cases as f64);
            j.key("cells").raw("[");
            for (k, c) in r.cells.iter().enumerate() {
                if k > 0 {
                    j.raw(",");
                }
                j.raw("{");
                j.num_field("line", c.line as f64);
                j.str_field("column", &c.column);
                j.str_field("typed", &c.typed);
                j.str_field("said", &c.said);
                j.raw("}");
            }
            j.raw("]");
        }
    }
    j.raw("}");
}

/// What a formula typed was read as (F8-a3), beside what the dry run found:
/// the method text it becomes, each input found with its unit, each spelling
/// written back, a note, and every finding by the formula's own line.
fn formula_field(j: &mut Json, file: &vleo_files::model::File, e: &Edits) {
    use vleo_sheet::method::formula::Found;
    let Some(fm) = e.edits.iter().find_map(|x| match x {
        Edit::Formula(fm) => Some(fm),
        _ => None,
    }) else {
        return;
    };
    let Some(uid) = file.blocks.first().map(|b| b.uid.clone()) else {
        return;
    };
    j.key("formula").raw("{");
    match edit::formula_of(file, &uid, fm) {
        Err(why) => {
            j.bool_field("ok", false);
            j.str_field("typed", &fm.typed);
            j.key("method").raw("null");
            j.key("inputs").raw("[]");
            j.key("rewrites").raw("[]");
            j.key("note").raw("null");
            j.key("diags").raw("[");
            j.raw("{");
            j.num_field("line", 0.0);
            j.str_field("severity", "error");
            j.str_field("msg", why.message());
            j.raw("}]");
        }
        Ok(r) => {
            j.bool_field("ok", r.ok());
            j.str_field("typed", &r.typed);
            match &r.method {
                Some(m) => j.str_field("method", m),
                None => j.key("method").raw("null"),
            };
            j.key("inputs").raw("[");
            for (k, x) in r.inputs.iter().enumerate() {
                if k > 0 {
                    j.raw(",");
                }
                j.raw("{");
                j.str_field("name", &x.name);
                match &x.port {
                    Some(p) => j.str_field("port", p),
                    None => j.key("port").raw("null"),
                };
                j.str_field(
                    "found",
                    match x.found {
                        Found::Name => "name",
                        Found::Symbol => "symbol",
                        Found::Missing => "missing",
                    },
                );
                j.str_field("unit", &x.unit);
                match &x.offered {
                    Some(o) => j.str_field("offered", o),
                    None => j.key("offered").raw("null"),
                };
                j.raw("}");
            }
            j.raw("]");
            j.key("rewrites").raw("[");
            for (k, w) in r.rewrites.iter().enumerate() {
                if k > 0 {
                    j.raw(",");
                }
                j.raw(&json::string(w));
            }
            j.raw("]");
            match &r.note {
                Some(n) => j.str_field("note", n),
                None => j.key("note").raw("null"),
            };
            j.key("diags").raw("[");
            for (k, d) in r.diags.iter().enumerate() {
                if k > 0 {
                    j.raw(",");
                }
                j.raw("{");
                j.num_field("line", d.line as f64);
                j.str_field(
                    "severity",
                    if d.severity == vleo_sheet::method::Severity::Error {
                        "error"
                    } else {
                        "warning"
                    },
                );
                j.str_field("msg", &d.msg);
                j.raw("}");
            }
            j.raw("]");
        }
    }
    j.raw("}");
}

/// The edits a request sends, from its fields.
fn edits_of(params: &str) -> Edits {
    let given = |k: &str| param(params, k).map(decode);
    let mut edits = Vec::new();
    if let Some(q) = given("question") {
        edits.push(Edit::Question(q));
    }
    for (k, part) in [
        ("explain_simply", Part::Simply),
        ("explain_breaks", Part::Breaks),
        ("explain_wrong", Part::Wrong),
    ] {
        if let Some(v) = given(k) {
            edits.push(Edit::Explain(part, v));
        }
    }
    if let Some(m) = given("method_text") {
        edits.push(Edit::Method(m));
    }
    if let Some(c) = given("case_csv") {
        edits.push(Edit::Cases(c));
    }
    // A table pasted is the cases, laid before a formula is read, as cases
    // typed are: whether a formula publishes beside its answer is read from
    // the cases it will run on.
    if let Some(text) = given("case_paste") {
        edits.push(Edit::Table(edit::Pasted {
            text,
            origin: given("case_paste.origin").unwrap_or_default(),
        }));
    }
    if let Some(typed) = given("formula") {
        edits.push(Edit::Formula(formula_of(params, typed)));
    }
    let declared = ["author", "ai", "source", "checked_by"]
        .iter()
        .any(|k| given(&format!("declared.{k}")).is_some())
        .then(|| {
            let g = |k: &str| given(&format!("declared.{k}")).unwrap_or_default();
            Declared {
                author: g("author"),
                ai: g("ai"),
                source: g("source"),
                checked_by: g("checked_by"),
            }
        });
    let fields = [
        "believed",
        "tested",
        "learned",
        "changed",
        "risks",
        "cost",
        "rests_on",
        "breaks_if",
    ];
    let why = fields
        .iter()
        .any(|k| given(&format!("why.{k}")).is_some())
        .then(|| {
            let g = |k: &str| given(&format!("why.{k}")).unwrap_or_default();
            vleo_sheet::template::Derisk {
                believed: g("believed"),
                tested: g("tested"),
                learned: g("learned"),
                changed: g("changed"),
                risks: g("risks"),
                cost: g("cost"),
                rests_on: g("rests_on"),
                breaks_if: g("breaks_if"),
            }
        });
    Edits {
        edits,
        declared,
        why,
    }
}

fn edited(id: &str, params: &str, ctx: &Ctx) -> Result<String, String> {
    me::carries_session(params)?;
    // One writer per file is asked even of a dry run: without a key there is
    // nobody whose writing it would be.
    if me::with_key(|_, _| ()).is_none() {
        return Err("no key is open: open yours, and the node is written as you".into());
    }
    let id = node_id(id)?;
    let dry = param(params, "dry_run")
        .map(decode)
        .is_some_and(|v| !v.is_empty() && v != "0");
    let copy = the_copy(ctx)?;
    let opened = param(params, "opened").map(decode).unwrap_or_default();
    let path = find(&copy, id)?;
    let e = edits_of(params);
    let group = group_of(&copy, &path)?;
    let group = &group;
    let at = results::now_utc();
    let tried = me::with_key(|key, me| {
        group
            .key(&me.name, &key.public().to_hex(), &at)
            .map_err(|e| e.message().to_string())?;
        edit::try_edit(&copy, &path, &opened, &e, &me.name, &at, APP, group)
            .map(|t| (t, me.name.clone()))
            .map_err(|e| e.message().to_string())
    })
    .ok_or("no key is open: open yours, and the node is written as you")??;
    let mut j = Json::new();
    j.raw("{");
    match tried {
        (
            Tried::SecondWriter {
                writer,
                revision,
                digest,
            },
            _,
        ) => {
            j.bool_field("ok", false);
            j.bool_field("conflict", true);
            j.bool_field("dry_run", dry);
            j.bool_field("written", false);
            j.str_field("node", id);
            j.str_field("writer", &writer);
            j.num_field("revision", revision as f64);
            j.str_field("opened", &digest);
            j.str_field(
                "message",
                &format!(
                    "{id} changed in your copy since you opened it: {writer} saved revision {revision}. Nothing is written; what you typed is kept: open it again, and write on what is there."
                ),
            );
        }
        (
            Tried::Would {
                file,
                revision,
                refused,
                checked,
                findings,
                report,
                about,
            },
            by,
        ) => {
            // A lesson is checked against the rows it names, as a save's is.
            let refused = refused.or_else(|| {
                lesson_holds(ctx, id, &file)
                    .err()
                    .map(|why| format!("{why}. Nothing is saved."))
            });
            let mut written = None;
            if !dry && refused.is_none() {
                let saved = me::with_key(|key, me| {
                    // Counted again for the write itself, as a save counts it:
                    // never left to the dry run before it.
                    group
                        .key(&me.name, &key.public().to_hex(), &at)
                        .map_err(|e| e.message().to_string())?;
                    edit::edit_in_place(&copy, &path, &opened, &e, &me.name, &at, APP, group)
                        .map_err(|e| e.message().to_string())
                })
                .ok_or("no key is open: open yours, and the node is written as you")??;
                written = Some(saved);
            }
            match written {
                Some(Saved::SecondWriter {
                    writer,
                    revision,
                    digest,
                    ..
                }) => {
                    j.bool_field("ok", false);
                    j.bool_field("conflict", true);
                    j.bool_field("dry_run", false);
                    j.bool_field("written", false);
                    j.str_field("node", id);
                    j.str_field("writer", &writer);
                    j.num_field("revision", revision as f64);
                    j.str_field("opened", &digest);
                    j.str_field(
                        "message",
                        &format!(
                            "{id} changed in your copy since you opened it: {writer} saved revision {revision}. Nothing is written; what you typed is kept."
                        ),
                    );
                }
                Some(Saved::Written {
                    revision, digest, ..
                }) => {
                    j.bool_field("ok", true);
                    j.bool_field("dry_run", false);
                    j.bool_field("written", true);
                    j.str_field("node", id);
                    j.str_field("path", &path);
                    j.num_field("revision", revision as f64);
                    j.str_field("opened", &digest);
                    tried_fields(&mut j, &None, true, &findings, &report, &about);
                    formula_field(&mut j, &file, &e);
                    table_field(&mut j, &file, &e);
                    j.str_field(
                        "message",
                        &format!(
                            "{id} is saved as revision {revision}, written by {by}, in your copy. Not signed: sign it once its checks hold."
                        ),
                    );
                }
                None => {
                    j.bool_field("ok", refused.is_none());
                    j.bool_field("dry_run", dry);
                    j.bool_field("written", false);
                    j.str_field("node", id);
                    j.str_field("path", &path);
                    j.num_field("revision", revision as f64);
                    j.str_field("opened", &opened);
                    tried_fields(&mut j, &refused, checked, &findings, &report, &about);
                    formula_field(&mut j, &file, &e);
                    table_field(&mut j, &file, &e);
                    j.str_field(
                        "message",
                        &match (&refused, dry) {
                            (Some(why), _) => why.clone(),
                            (None, true) => format!(
                                "Checked, not written: saved now, {id} would be revision {revision}."
                            ),
                            (None, false) => unreachable!("a save that is not refused is written"),
                        },
                    );
                }
            }
        }
    }
    j.raw("}");
    Ok(j.0)
}

/// What a dry run found, as the screen shows it beside what was typed.
fn tried_fields(
    j: &mut Json,
    refused: &Option<String>,
    checked: bool,
    findings: &vleo_files::checks::Findings,
    report: &Option<(Vec<vleo_sheet::method::Case>, vleo_sheet::method::Report)>,
    about: &[&str],
) {
    j.str_field("refused", refused.as_deref().unwrap_or(""));
    // Whether what was typed was checked: when its content is refused, no
    // finding or case is given in its place.
    j.bool_field("checked", checked);
    findings_field(j, findings);
    j.key("about").raw("[");
    for (i, a) in about.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw(&json::string(a));
    }
    j.raw("]");
    j.bool_field("needs_record", !about.is_empty());
    j.key("method").raw("{");
    j.bool_field("runs", report.is_some());
    j.key("diags").raw("[");
    if let Some((_, r)) = report {
        for (i, d) in r.diags.iter().enumerate() {
            if i > 0 {
                j.raw(",");
            }
            j.raw("{");
            j.num_field("line", d.line as f64);
            j.str_field(
                "severity",
                if d.severity == vleo_sheet::method::Severity::Error {
                    "error"
                } else {
                    "warning"
                },
            );
            j.str_field("msg", &d.msg);
            j.raw("}");
        }
    }
    j.raw("]");
    j.key("cases").raw("[");
    if let Some((_, r)) = report {
        for (i, (c, v)) in r.cases.iter().enumerate() {
            if i > 0 {
                j.raw(",");
            }
            j.raw("{");
            j.str_field("label", &c.label);
            j.bool_field("refuses", c.refuses());
            j.bool_field("agrees", v.agrees());
            // The case's value is its reference, not the code's.
            j.str_field("says", &v.text(c).replace("your code", "the case"));
            j.raw("}");
        }
    }
    j.raw("]}");
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
