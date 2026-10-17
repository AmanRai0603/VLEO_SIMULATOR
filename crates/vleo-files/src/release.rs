//! A group's release, sealed by its subsystem engineer (docs/briefs/F5.md, the
//! seal; W5).
//!
//! The release is the group's file and every node file of its branch, as one
//! file of kind `group release`. It is sealed only when every assigned node is
//! there, signed by its node engineer as it is now and at its current
//! contract, with its own checks passing, and the group's file opens through
//! the chain. Anything short of that is refused, every reason named: a seal is
//! never a release with a node quietly left out.
//!
//! The release records what it was built on: the released design its group
//! file is `based_on`, with its `base_version`, and the release before it. Its
//! fingerprint is the digest of what it holds, and the subsystem engineer, or
//! their deputy, signs it with a key the programme's file counts for them. The
//! sealed file is then held to [`chain::check_release_in`], the check today's
//! design makes of it, before it is written: what is sealed here is what will
//! be taken there.
//!
//! [`seal_in_place`] writes it into the copy's `releases/` folder under the
//! copy's lock, never over a release already there: a release is never edited.

use crate::chain::{self, Registry, FILE};
use crate::checks;
use crate::error::{Error, ErrorKind};
use crate::keys::SigningKey;
use crate::meta::{Kind, ReleaseVersion};
use crate::model::File;

fn refused(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Malformed, message)
}

/// What every node file of a branch must be for the branch to be sealed, each
/// failing node named with every reason. Empty when nothing stands in the way.
pub fn blockers(group: &File, nodes: &[File], registry: &Registry) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for n in nodes {
        let uid = n.meta.get("block_uid").map_or("", String::as_str);
        let name = n
            .blocks
            .iter()
            .find(|b| b.uid == uid)
            .map_or(uid, |b| b.id.as_str());
        if n.blocks.len() != 1 || n.blocks[0].uid != uid {
            out.push(format!(
                "{name}: a node file holds its one block, and this holds {}",
                n.blocks.len()
            ));
            continue;
        }
        if seen.contains(&uid) {
            out.push(format!(
                "{name}: two node files hold it; its node engineer keeps one"
            ));
            continue;
        }
        seen.push(uid);
        if group.blocks.iter().any(|b| b.uid == uid) || holds_rows_of(group, uid) {
            out.push(format!(
                "{name}: the group file holds rows of it too, so the release cannot hold \
                 its node file as signed; it is issued again first"
            ));
            continue;
        }
        let Some(a) = group.assignments.iter().find(|a| a.block_uid == uid) else {
            out.push(format!(
                "{name}: the group file assigns it to nobody, so nobody's signature counts for it"
            ));
            continue;
        };
        if n.blocks[0].contract_version < a.contract_version {
            out.push(format!(
                "{name}: behind its contract — written at contract {}, and the group's is {}: \
                 its node engineer signs it again at the new contract",
                n.blocks[0].contract_version, a.contract_version
            ));
            continue;
        }
        if let Err(e) = chain::check_node(n, uid, registry) {
            out.push(format!("{name}: not signed as it is — {}", e.message()));
            continue;
        }
        let found = crate::checks::check_node(n);
        let errors: Vec<String> = found
            .errors()
            .map(|e| format!("{}: {}", e.place, e.what))
            .collect();
        if !errors.is_empty() {
            out.push(format!(
                "{name}: its checks refuse it — {}",
                errors.join("; ")
            ));
        }
    }
    for a in &group.assignments {
        if !nodes
            .iter()
            .any(|n| n.meta.get("block_uid").map(String::as_str) == Some(a.block_uid.as_str()))
        {
            out.push(format!(
                "{}: assigned to {}, and there is no node file of it",
                a.block_uid, a.person
            ));
        }
    }
    out
}

/// Whether `group` holds a row belonging to the block `uid`.
fn holds_rows_of(group: &File, uid: &str) -> bool {
    group.ports.iter().any(|p| p.block_uid == uid)
        || group.wires.iter().any(|w| w.to_block == uid)
        || group.mounts.iter().any(|m| m.block_uid == uid)
        || group.closures.iter().any(|c| c.block_uid == uid)
        || group.loops.iter().any(|l| l.block_uid == uid)
        || group.cases.iter().any(|c| c.block_uid == uid)
        || group.texts.iter().any(|t| t.scope == uid)
        || group.tables.iter().any(|t| t.scope == uid)
        || group.media.iter().any(|m| m.scope == uid)
}

/// The group `group`'s release `version`, sealed by `by` with `key` at `at`:
/// its group file and every node file in `nodes`, after `previous` (the
/// release before it, if any). Refused, with every reason, unless `by` frames
/// the group ([`chain::may_frame`]), the group file opens through the chain,
/// the version is a release's and comes after `previous`, and nothing in
/// [`blockers`] stands in the way. The sealed release is held to
/// [`chain::check_release_in`] before it is given back.
#[allow(clippy::too_many_arguments)]
pub fn seal(
    group: &File,
    nodes: &[File],
    version: &str,
    previous: Option<&str>,
    by: &str,
    key: &SigningKey,
    at: &str,
    app: &str,
    mounted_by: &Registry,
    programme: &Registry,
) -> Result<File, Error> {
    if group.kind()? != Kind::Group {
        return Err(refused(format!(
            "a release is sealed from a group file, not a {}",
            group.kind()?
        )));
    }
    let group_id = group.meta.get("group_id").cloned().unwrap_or_default();
    chain::may_frame(
        &group_id,
        by,
        &key.public().to_hex(),
        at,
        mounted_by,
        programme,
    )?;
    let v = ReleaseVersion::parse(version).ok_or_else(|| {
        refused(format!(
            "{version:?} is not a release's version: major.minor, such as 1.2"
        ))
    })?;
    if let Some(p) = previous {
        let pv = ReleaseVersion::parse(p)
            .ok_or_else(|| refused(format!("the release before, {p:?}, is not a version")))?;
        if v <= pv {
            return Err(refused(format!(
                "{group_id} {version} does not come after {p}, the release before it: \
                 a release is never sealed again under a version already taken"
            )));
        }
    }
    let registry = chain::open_group_in(group, mounted_by, programme)
        .map_err(|e| refused(format!("{group_id}'s group file: {}", e.message())))?;
    let blocking = blockers(group, nodes, &registry);
    if !blocking.is_empty() {
        return Err(refused(format!(
            "{group_id} {version} is not sealed ({} reason{}): {}",
            blocking.len(),
            if blocking.len() == 1 { "" } else { "s" },
            blocking.join("; ")
        )));
    }

    let mut r = File::new(Kind::GroupRelease, app);
    r.meta.insert("group_id".into(), group_id.clone());
    r.meta.insert("version".into(), version.to_string());
    r.meta
        .insert("previous".into(), previous.unwrap_or("").to_string());
    r.meta.insert(
        "based_on".into(),
        group.meta.get("based_on").cloned().unwrap_or_default(),
    );
    if let Some(b) = group.meta.get("base_version") {
        r.meta.insert("base_version".into(), b.clone());
    }
    r.meta.insert("writer".into(), by.to_string());
    // The group's own content and record. Its file's signature is for the
    // group file, not the release: the seal below is the release's.
    r.people = group.people.clone();
    r.keys = group.keys.clone();
    r.assignments = group.assignments.clone();
    r.blocks = group.blocks.clone();
    r.ports = group.ports.clone();
    r.wires = group.wires.clone();
    r.mounts = group.mounts.clone();
    r.closures = group.closures.clone();
    r.loops = group.loops.clone();
    r.cases = group.cases.clone();
    r.texts = group.texts.clone();
    r.tables = group.tables.clone();
    r.media = group.media.clone();
    r.changes = group.changes.clone();
    r.requests = group.requests.clone();
    r.comments = group.comments.clone();
    r.signatures = group
        .signatures
        .iter()
        .filter(|s| s.scope != FILE)
        .cloned()
        .collect();
    for n in nodes {
        r.blocks.extend(n.blocks.iter().cloned());
        r.ports.extend(n.ports.iter().cloned());
        r.wires.extend(n.wires.iter().cloned());
        r.closures.extend(n.closures.iter().cloned());
        r.loops.extend(n.loops.iter().cloned());
        r.cases.extend(n.cases.iter().cloned());
        r.texts.extend(n.texts.iter().cloned());
        r.tables.extend(n.tables.iter().cloned());
        r.media.extend(n.media.iter().cloned());
        r.changes.extend(n.changes.iter().cloned());
        r.requests.extend(n.requests.iter().cloned());
        r.comments.extend(n.comments.iter().cloned());
        r.signatures
            .extend(n.signatures.iter().filter(|s| s.scope != FILE).cloned());
    }
    let fingerprint = chain::digest(&r, FILE)?;
    r.meta.insert("fingerprint".into(), fingerprint);
    r.meta.insert("sealed_by".into(), by.to_string());
    r.meta.insert("sealed_at".into(), at.to_string());
    r.check_values()?;
    // Every check today's design makes of a release's content: its nodes,
    // its requirements, and the record of which belief broke.
    let found = checks::check_release(&r);
    let errors: Vec<String> = found
        .errors()
        .map(|e| format!("{}: {}", e.place, e.what))
        .collect();
    if !errors.is_empty() {
        return Err(refused(format!(
            "{group_id} {version} is not sealed: its checks refuse it — {}",
            errors.join("; ")
        )));
    }
    let row = chain::sign(&r, FILE, by, key, at, "ok", "")?;
    r.signatures.push(row);

    // What today's design will ask of it, asked now.
    let checked = chain::check_release_in(&r, mounted_by, programme)?;
    if !checked.holds() {
        return Err(refused(format!(
            "{group_id} {version} was sealed and does not check through the chain, so it is \
             not given: {}",
            checked.refused.join("; ")
        )));
    }
    Ok(r)
}

/// The releases already in `dir`, each `(version, file name)`, newest first:
/// every `.vleo` that opens as a release of `group_id`. One that does not
/// open, or is another group's, refuses the seal, by its name: a version is
/// never chosen beside a release nobody can read.
#[cfg(not(target_arch = "wasm32"))]
pub fn releases_in(
    dir: &std::path::Path,
    group_id: &str,
    app: &str,
) -> Result<Vec<(ReleaseVersion, String)>, Error> {
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    let io = |e: std::io::Error| Error::new(ErrorKind::Io, format!("{}: {e}", dir.display()));
    for e in std::fs::read_dir(dir).map_err(io)? {
        let p = e.map_err(io)?.path();
        if p.extension().is_none_or(|x| x != "vleo") {
            continue;
        }
        let name = p
            .file_name()
            .map_or(String::new(), |n| n.to_string_lossy().into_owned());
        let how = crate::upgrade::Upgrade { app, at: "" };
        let f = crate::sqlite::open(&p, &how)
            .map_err(|e| refused(format!("releases/{name} cannot be read: {}", e.message())))?
            .file;
        if f.meta.get("group_id").map(String::as_str) != Some(group_id) {
            return Err(refused(format!(
                "releases/{name} is not a release of {group_id}"
            )));
        }
        let v = f
            .meta
            .get("version")
            .and_then(|v| ReleaseVersion::parse(v))
            .ok_or_else(|| refused(format!("releases/{name} says no release version")))?;
        out.push((v, name));
    }
    out.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(out)
}

/// `group_id`'s release `version`, sealed from the copy at `copy` and written
/// into `groups/<group_id>/releases/<group_id>-<version>.vleo`, under the
/// copy's lock from the first read to the write. The release before it is the
/// newest already there. Refused, writing nothing, for anything [`seal`]
/// refuses, and when a file of that name is already there: a release is never
/// written over. Gives the path written, inside the copy.
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
pub fn seal_in_place(
    copy: &std::path::Path,
    group_id: &str,
    version: &str,
    by: &str,
    key: &SigningKey,
    at: &str,
    app: &str,
    mounted_by: &Registry,
    programme: &Registry,
) -> Result<String, Error> {
    if !crate::breakdown::valid_id(group_id) {
        return Err(refused(format!("{group_id:?} is not a group's id")));
    }
    let _held = crate::save::lock(copy)?;
    let (_, group) = crate::breakdown::group_file_in(copy, group_id)?;
    let how = crate::upgrade::Upgrade { app, at };
    let nodes = crate::breakdown::nodes_in(copy, group_id, &how)?;
    let dir = copy.join("groups").join(group_id).join("releases");
    let before = releases_in(&dir, group_id, app)?;
    let previous = before.first().map(|(v, _)| v.to_string());
    let rel = format!("groups/{group_id}/releases/{group_id}-{version}.vleo");
    let path = copy.join(&rel);
    if path.exists() {
        return Err(refused(format!(
            "{rel} is already there: a release is never written over"
        )));
    }
    let r = seal(
        &group,
        &nodes,
        version,
        previous.as_deref(),
        by,
        key,
        at,
        app,
        mounted_by,
        programme,
    )?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", dir.display())))?;
    crate::sqlite::write(&r, &path)?;
    Ok(rel)
}
