//! A group started by its owner, once the systems' file mounts it
//! (docs/briefs/F6.md, F6-a; W1; F6's question 20).
//!
//! The system engineer mounts a group and names its owner (`frame`); the
//! owner, or their deputy, starts its file: `groups/<g>/<g>.vgroup`, new,
//! so one writer signs it. It holds its meta, its base copied from the
//! systems' file, one top heading whose id is the group's and whose question
//! is the mount's, and a change row saying it was started. It opens through
//! the chain as it will be read (`chain::open_group_in`), or nothing is
//! written. The mount's ports are not copied: what the group answers is the
//! mount's, read from the systems' file.
//!
//! Not here: registering the group's node engineers (People, F1), framing
//! its branch (`breakdown`), its file reaching the drive (F5's put-back), and
//! its release in today's design (F5).

use crate::breakdown::valid_id;
use crate::chain::{self, Registry, FILE};
use crate::error::{Error, ErrorKind};
use crate::keys::SigningKey;
use crate::meta::Kind;
use crate::model::{Block, Change, File};

fn refused(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Malformed, message)
}

/// The group file of `group`, started as `by` with `key` at `at`: mounted
/// by `systems`, the systems' file as it opens through the chain, and
/// `programme`, the programme's. `files` are every other file of the design,
/// where the block it is mounted on may be held (W13). Refused by name, with
/// nothing given back.
#[allow(clippy::too_many_arguments)]
pub fn start(
    group: &str,
    systems: &Registry,
    programme: &Registry,
    files: &[File],
    by: &str,
    key: &SigningKey,
    at: &str,
    app: &str,
) -> Result<File, Error> {
    if !valid_id(group) || group == "programme" || group == "systems" {
        return Err(refused(format!(
            "{group:?} is not a subsystem's group: the programme's and the systems' files are not started here"
        )));
    }
    let mount = systems
        .file
        .mounts
        .iter()
        .find(|m| m.group_id == group)
        .ok_or_else(|| {
            refused(format!(
                "the systems' file mounts no group {group}: the system engineer mounts it, and names its owner, first"
            ))
        })?;
    if crate::checks::is_assistant(by) {
        return Err(refused(format!(
            "{by} is an assistant's name: a group is started by a person"
        )));
    }
    let owner = chain::may_frame(group, by, &key.public().to_hex(), at, systems, programme)
        .map_err(|e| {
            refused(format!(
                "{} — the owner the mount names, or their deputy, starts {group}",
                e.message()
            ))
        })?
        .to_string();
    let started = files.iter().any(|f| {
        f.kind().ok() == Some(Kind::Group)
            && f.meta.get("group_id").map(String::as_str) == Some(group)
    });
    if started {
        return Err(refused(format!(
            "{group} is started already: its group file is in groups/{group}/"
        )));
    }
    let taken = std::iter::once(&systems.file)
        .chain(std::iter::once(&programme.file))
        .chain(files.iter())
        .any(|f| f.blocks.iter().any(|b| b.uid == group));
    if taken {
        return Err(refused(format!(
            "{group} is a block's uid in the design already, so it cannot be its heading's"
        )));
    }
    // The question the group answers is the mount's, wherever its block is
    // held: the systems' own leaf, or a leaf of a subsystem's branch.
    let question = std::iter::once(&systems.file)
        .chain(files.iter())
        .flat_map(|f| f.blocks.iter())
        .find(|b| b.uid == mount.block_uid)
        .map(|b| b.question.trim().to_string())
        .ok_or_else(|| {
            refused(format!(
                "{group} is mounted on {}, which no branch of the copy holds, so the question it answers is not known: the system engineer mounts it again",
                mount.block_uid
            ))
        })?;
    let mut f = File::new(Kind::Group, app);
    f.meta.insert("group_id".into(), group.into());
    f.meta.insert("writer".into(), owner.clone());
    for k in ["based_on", "base_version"] {
        if let Some(v) = systems.file.meta.get(k) {
            f.meta.insert(k.into(), v.clone());
        }
    }
    f.meta.entry("based_on".into()).or_default();
    f.blocks.push(Block {
        uid: group.into(),
        id: group.into(),
        parent_uid: String::new(),
        question,
        behaviour: "children".into(),
        perspective: "subsystem".into(),
        ord: 0,
        archived: 0,
        contract_version: 1,
        revision: 0,
    });
    f.changes.push(Change {
        at: at.into(),
        who: by.into(),
        scope: group.into(),
        what: format!(
            "start: {group} begun by {by}, for its owner {owner}, as the systems' file mounts it on {}",
            mount.block_uid
        ),
        before: None,
        after: Some(format!(
            "block uid={group:?} id={group:?} behaviour=\"children\" question={:?}\n",
            f.blocks[0].question
        )),
    });
    let row = chain::sign(&f, FILE, by, key, at, "ok", "")?;
    f.signatures.push(row);
    f.check_meta()?;
    chain::open_group_in(&f, systems, programme).map_err(|e| {
        refused(format!(
            "{group}'s new file would not open through the chain, so it is not started: {}",
            e.message()
        ))
    })?;
    Ok(f)
}

/// Whether `group`, mounted, is still to be started in the copy: no group
/// file, and a folder that is absent or holds nothing, the one state
/// `start_in_place` starts from. A folder holding anything else is not
/// this, and is said as what it is.
#[cfg(not(target_arch = "wasm32"))]
pub fn to_start(copy: &std::path::Path, group: &str) -> bool {
    let dir = copy.join("groups").join(group);
    !dir.join(format!("{group}.vgroup")).exists()
        && match std::fs::read_dir(&dir) {
            Ok(mut d) => d.next().is_none(),
            Err(e) => e.kind() == std::io::ErrorKind::NotFound,
        }
}

/// `group` started in the copy, under the copy's lock from the first read
/// to the rename, as an edit is (`breakdown::edit_in_place`): its folder
/// holds nothing yet, the systems' file is read and opened through
/// `programme` under the lock, and every other file of the copy is read
/// there. Gives the new file's digest.
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
pub fn start_in_place(
    copy: &std::path::Path,
    group: &str,
    by: &str,
    key: &SigningKey,
    at: &str,
    app: &str,
    programme: &Registry,
) -> Result<String, Error> {
    if !valid_id(group) {
        return Err(refused(format!("{group:?} is not a group's id")));
    }
    if group == "programme" || group == "systems" {
        return Err(refused(format!(
            "{group:?} is not a subsystem's group: the programme's and the systems' files are not started here"
        )));
    }
    let _held = crate::save::lock(copy)?;
    let dir = copy.join("groups").join(group);
    let at_file = dir.join(format!("{group}.vgroup"));
    if at_file.exists() {
        return Err(refused(format!(
            "{group} is started already: groups/{group}/{group}.vgroup is in your copy"
        )));
    }
    if dir.exists() {
        let io = |e: std::io::Error| Error::new(ErrorKind::Io, format!("{}: {e}", dir.display()));
        if let Some(e) = std::fs::read_dir(&dir).map_err(io)?.next() {
            let name = e.map_err(io)?.file_name().to_string_lossy().into_owned();
            return Err(refused(format!(
                "groups/{group}/ holds {name} already: a group is started in a folder that holds nothing"
            )));
        }
    }
    let how = crate::upgrade::Upgrade { app, at };
    let (_, sf) = crate::breakdown::group_file_in(copy, "systems").map_err(|e| {
        refused(format!(
            "the systems' file cannot be read in your copy, so no group is started from it: {}",
            e.message()
        ))
    })?;
    let systems = chain::open_systems(&sf, programme).map_err(|e| {
        refused(format!(
            "the systems' file does not open through the chain, so no group is started from it: {}",
            e.message()
        ))
    })?;
    let files = crate::breakdown::others_in(copy, group, &how)?;
    let f = start(group, &systems, programme, &files, by, key, at, app)?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", dir.display())))?;
    let written = crate::sqlite::written(&f, &at_file)?;
    Ok(crate::copied::digest(&written))
}
