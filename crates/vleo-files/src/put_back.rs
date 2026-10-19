//! A person's own work put back on the shared drive from their working copy
//! (docs/briefs/F3.md, F3-c; docs/PLAN_F.md, decision 3).
//!
//! The copy's record (`copied.toml`) says what every file was when the copy
//! was taken. A file whose bytes in the copy are not that is the person's own
//! work, and it goes back only when all of these hold:
//!
//! - it differs from the drive, and the drive holds a file at that path: a
//!   put-back changes the files there are, never makes or deletes one;
//! - the drive's bytes are still the ones the record holds: a file someone
//!   else changed on the drive since the copy is never written over;
//! - it is a node's file, the same file as the drive's, changed only as its
//!   writer may change it: it holds the same node as the drive's file at
//!   that path, changes nothing outside the node's own rows (as a save is
//!   held, `save`), keeps every signature the drive's file holds and adds
//!   only the person's own, is one revision with its block, is not older than
//!   the drive's (and at the same revision differs from it only by that
//!   signature and the person's requests), says a writer who may write it,
//!   and its own checks hold;
//! - it carries a signature by the person that counts through the chain as
//!   the **drive** holds it now, made with a key the drive's files still
//!   count for them at the moment it goes back.
//!
//! Or it is the drive's file, at the same revision with the same signatures
//! of its node, and only asks (F3-d): it adds requests on the node, each the
//! person's own, open and signed by them as the drive counts it now. A node
//! that cannot be signed yet still asks. In either case every request the
//! drive's file holds, and every signature of one, is kept as it is.
//!
//! Or it is a group's release, sealed in the copy (F5): a new file in the
//! group's `releases/`, named for the group and the version it says, sealed
//! by the person putting it back with a key the drive counts for them now,
//! after the drive's latest release of the group, and checking through the
//! chain as the drive holds it now: its seal through the file that mounts the
//! group, every node in it through the group file it carries. A release is
//! never written over, so one already on the drive at that path stays.
//!
//! Or it is a released design (F6-b): a new file in `design/`, named for the
//! version it says, released by the person putting it back, following the
//! drive's latest, and checking through the chain as the drive holds it now,
//! signed with a key that counts then; one at a time. Its record,
//! `design/NOTES.md` and the files of `readable/`, goes back by the system
//! engineer or their deputy beside it, over the drive's only when the
//! drive's is as copied.
//!
//! A group's file does not go back yet: what may change in it at its valve
//! is not yet checked against the drive's file, so a change of its people
//! stays in the copy.
//!
//! Every other changed file is named, with the reason. Nothing is deleted on
//! the drive, and a file gone from the copy is said, not followed.
//!
//! [`put_back`] answers the report and writes nothing, until it is given the
//! files to put back, each with the digest the report gave it, and those
//! exactly the ones the report gives as it is made again. That second report
//! is made under the drive's lock, from the chain opened again under it, so
//! what is written is judged against the drive as it is when it is written.
//! Each file is written to a name of the program's own beside its place, then
//! renamed over it; the copy's record then says the file is as the drive
//! holds it. A link anywhere on the way to a file on the drive is refused.
//!
//! The locks are of files: the copy's beside it ([`crate::save::lock`]), the
//! drive's in its own folder. Two programs that put back through this
//! library never write one file at once; a sync client or a person who
//! writes the drive by hand is not held by them.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::chain::{self, Registry};
use crate::checks;
use crate::copied::{self, Copied};
use crate::error::{Error, ErrorKind};
use crate::meta::Kind;
use crate::model::File;
use crate::release;
use crate::save;

/// The drive's chain, opened from the drive's own files: the anchor START
/// HERE names, the programme's file it opened, the systems' file and every
/// group's file that opened through it, and why each other was left out.
pub struct Chain {
    pub anchor: String,
    pub programme: Registry,
    pub systems: Option<Registry>,
    pub groups: Vec<Registry>,
    pub left_out: Vec<String>,
}

/// One changed file: whether it goes back, and why, or why not, with the
/// SHA-256 of its bytes in the copy (empty when it is gone from the copy).
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub path: String,
    pub back: bool,
    pub why: String,
    pub digest: String,
}

/// What a put-back found, file by file, whether it wrote, and the files
/// already on the drive as the copy holds them that the copy's record now
/// says so of.
#[derive(Debug, Default)]
pub struct Report {
    pub lines: Vec<Line>,
    pub written: bool,
    pub recorded: Vec<String>,
}

/// The line of a file already on the drive as the copy holds it.
const ALREADY: &str = "already on the drive as it is in your copy";

impl Report {
    /// The paths that go back, in order.
    pub fn back(&self) -> Vec<String> {
        self.lines
            .iter()
            .filter(|l| l.back)
            .map(|l| l.path.clone())
            .collect()
    }

    /// The files that go back, each with its digest: what to name to write
    /// them.
    pub fn named(&self) -> Vec<(String, String)> {
        self.lines
            .iter()
            .filter(|l| l.back)
            .map(|l| (l.path.clone(), l.digest.clone()))
            .collect()
    }
}

fn refused(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Malformed, message)
}

fn io(at: &Path, e: std::io::Error) -> Error {
    Error::new(ErrorKind::Io, format!("{}: {e}", at.display()))
}

/// The drive's lock, `.vleo-put-back.lock` in its own folder: held while one
/// program writes the drive, and let go when dropped, only if it is still
/// the one this program made.
struct DriveLock(PathBuf, String);

impl Drop for DriveLock {
    fn drop(&mut self) {
        if std::fs::read_to_string(&self.0).is_ok_and(|t| t.trim() == self.1) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
}

/// The names the program's own files carry: the drive's lock in its folder,
/// a file being written beside its place, and the copy's record being
/// written beside it.
const LOCK: &str = ".vleo-put-back.lock";
const INCOMING: &str = ".vleo-incoming-";
const RECORD: &str = ".vleo-record-";

/// A word of this program's own: its process and the moment.
fn token() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    format!("{}-{nanos}", std::process::id())
}

fn drive_lock(drive: &Path) -> Result<DriveLock, Error> {
    let at = drive.join(LOCK);
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&at)
    {
        Ok(mut f) => {
            use std::io::Write as _;
            let mine = token();
            writeln!(f, "{mine}").map_err(|e| io(&at, e))?;
            Ok(DriveLock(at, mine))
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Err(Error::new(
            ErrorKind::Io,
            format!(
                "someone is putting work back on the drive ({} is there): wait for them, or remove that file if nobody is",
                at.display()
            ),
        )),
        Err(e) => Err(io(&at, e)),
    }
}

/// Every file under `folder`, by its path inside it with `/` between parts,
/// and the SHA-256 of its bytes. A link, a key, anything but a folder or a
/// file, or a name the record could not hold is refused by name, never
/// followed or skipped. Left out: the record itself, and the program's own
/// files, each only as an ordinary file and only where the program puts it
/// (the drive's lock and the record being written at the top, a file being
/// written beside its place).
pub fn digests(folder: &Path) -> Result<BTreeMap<String, String>, Error> {
    let mut out = BTreeMap::new();
    let mut todo = vec![PathBuf::new()];
    while let Some(rel) = todo.pop() {
        let here = folder.join(&rel);
        for e in std::fs::read_dir(&here).map_err(|e| io(&here, e))? {
            let e = e.map_err(|e| io(&here, e))?;
            let name = e.file_name().to_string_lossy().into_owned();
            let path = rel.join(&name);
            let shown = path
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            let kind = e.file_type().map_err(|e| io(&folder.join(&path), e))?;
            if kind.is_symlink() {
                return Err(refused(format!(
                    "{shown} is a link: a database holds files, and a link is never followed"
                )));
            }
            if !kind.is_dir() && !kind.is_file() {
                return Err(refused(format!(
                    "{shown} is neither a folder nor a file: a database holds only those"
                )));
            }
            if name.to_ascii_lowercase().ends_with(".vkey") {
                return Err(refused(format!(
                    "{shown} is a key: a key lives on its person's own computer, never in a database"
                )));
            }
            if kind.is_file() && name.starts_with(INCOMING) {
                return Err(refused(format!(
                    "{shown} is a file being put back on the drive: someone is putting work back, or a put-back was cut short. Try again once it is done, or remove it if nobody is putting work back"
                )));
            }
            let top = rel.as_os_str().is_empty();
            let own = kind.is_file()
                && top
                && (name == copied::FILE_NAME || name == LOCK || name.starts_with(RECORD));
            if own {
                continue;
            }
            if !copied::path_ok(&shown) {
                return Err(refused(format!(
                    "{shown:?} is not a name the database can hold"
                )));
            }
            if kind.is_dir() {
                todo.push(path);
            } else {
                let at = folder.join(&path);
                let bytes = std::fs::read(&at).map_err(|e| io(&at, e))?;
                out.insert(shown, copied::digest(&bytes));
            }
        }
    }
    Ok(out)
}

/// The record beside the copy, refused when there is none.
fn record_of(copy: &Path) -> Result<Copied, Error> {
    let at = copy.join(copied::FILE_NAME);
    match std::fs::read_to_string(&at) {
        Ok(t) => Copied::from_toml(&t),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(refused(
            "there is no copy of the database here to put back from: copy it from the drive first",
        )),
        Err(e) => Err(io(&at, e)),
    }
}

/// Refused when any folder on the way to `path` on the drive, or the file
/// itself, is a link: what is read or written there is inside the drive.
fn no_link_on_the_way(drive: &Path, path: &str) -> Result<(), Error> {
    let mut at = drive.to_path_buf();
    for part in path.split('/') {
        at.push(part);
        match std::fs::symlink_metadata(&at) {
            Ok(m) if m.file_type().is_symlink() => {
                return Err(refused(format!(
                    "{path} on the drive is reached through a link, {}: a link is never followed, and nothing is written through one",
                    at.display()
                )))
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(io(&at, e)),
        }
    }
    Ok(())
}

/// The bytes of the file `path` names on the drive, if there is one: an
/// ordinary file, reached through no link. `None` when the drive holds none.
fn on_drive(drive: &Path, path: &str) -> Result<Option<Vec<u8>>, Error> {
    no_link_on_the_way(drive, path)?;
    let at = drive.join(path);
    match std::fs::symlink_metadata(&at) {
        Ok(m) if m.file_type().is_file() => std::fs::read(&at).map(Some).map_err(|e| io(&at, e)),
        Ok(_) => Err(refused(format!(
            "{path} on the drive is not an ordinary file: nothing is written over it"
        ))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(io(&at, e)),
    }
}

/// `f` with only `me`'s signatures of `scope`, and every other row as it is:
/// what counts through the chain from it counts because `me` signed it.
fn only_by(f: &File, scope: &str, me: &str) -> File {
    let mut only = f.clone();
    only.signatures
        .retain(|s| s.scope != scope || s.signer == me);
    only
}

/// `f` with only `me`'s signatures of `scope` made with a key `registry`
/// still counts for them at `at`, the moment the work goes back, and every
/// other row as it is: a key revoked since, from whatever day, signs nothing
/// that goes back.
fn only_counting_now(f: &File, scope: &str, me: &str, at: &str, registry: &Registry) -> File {
    let mut only = f.clone();
    only.signatures.retain(|s| {
        s.scope != scope || (s.signer == me && registry.key(me, &s.public_key, at).is_ok())
    });
    only
}

/// The registry the node files in `group`'s folder are read against: the
/// programme's, the systems' or the group's own, as the drive's chain opened
/// it, or why none is.
fn registry_of<'a>(chain: &'a Chain, group: &str) -> Result<&'a Registry, String> {
    let found = match group {
        "programme" => Some(&chain.programme),
        "systems" => chain.systems.as_ref(),
        _ => chain
            .groups
            .iter()
            .find(|r| r.file.meta.get("group_id").map(String::as_str) == Some(group)),
    };
    found.ok_or_else(|| {
        let why = chain
            .left_out
            .iter()
            .find(|w| w.starts_with(&format!("{group}: ")))
            .map_or(String::new(), |w| format!(": {w}"));
        format!("{group}'s file on the drive does not check through the chain, so no signature in its folder counts{why}")
    })
}

/// Whether the changed node file at `path`, `bytes` in the copy, goes back as
/// `me`'s work over `theirs`, the drive's file at that path.
fn node_verdict(
    path: &str,
    group: &str,
    bytes: &[u8],
    theirs: Option<&[u8]>,
    me: &str,
    at: &str,
    chain: &Chain,
) -> Result<String, String> {
    let said = |e: Error| e.message().to_string();
    let f = save::parsed(bytes).map_err(|e| format!("it does not read: {}", e.message()))?;
    save::the_node(&f, path).map_err(said)?;
    let Some(theirs) = theirs else {
        return Err(
            "the drive holds no node file at this path: a put-back changes the node files there are, and a new node's file comes from its group"
                .into(),
        );
    };
    let drive_file = save::parsed(theirs).map_err(|e| {
        format!(
            "the drive's file at this path does not read: {}",
            e.message()
        )
    })?;
    save::the_node(&drive_file, path).map_err(|e| {
        format!(
            "the drive's file at this path is not a node's: {}",
            e.message()
        )
    })?;
    let (mine, was) = (&f.blocks[0], &drive_file.blocks[0]);
    for key in ["file_kind", "group_id", "block_uid"] {
        if f.meta.get(key) != drive_file.meta.get(key) {
            return Err(format!(
                "it says it is {key} {:?}, and the drive's file at this path is {:?}: a put-back does not put one node's file in another's place",
                f.meta.get(key).map_or("", String::as_str),
                drive_file.meta.get(key).map_or("", String::as_str)
            ));
        }
    }
    if (&mine.id, &mine.parent_uid, mine.contract_version)
        != (&was.id, &was.parent_uid, was.contract_version)
    {
        return Err(
            "it changes the node's id, the block it hangs from or its contract's version: a node is not moved, and a contract changes by a request"
                .into(),
        );
    }
    let registry = registry_of(chain, group)?;
    // Where a group hangs, in whichever file mounts it: its own group's, or
    // the systems' file on a leaf of a subsystem's branch (W13).
    let files: Vec<&File> = std::iter::once(&chain.programme.file)
        .chain(chain.systems.iter().map(|r| &r.file))
        .chain(chain.groups.iter().map(|r| &r.file))
        .collect();
    if let Some((m, by)) = crate::frame::mounted_on_any(&mine.uid, &files) {
        return Err(format!(
            "{} is where a group hangs, {} mounted in {by}'s file, not a node: no node's file is put back for it",
            mine.id, m.group_id
        ));
    }
    f.check_meta().map_err(said)?;
    // The node's signatures: every one the drive's file holds is kept, and
    // the only ones added are `me`'s own.
    let rows = |x: &File| -> Vec<crate::model::SignatureRow> {
        x.signatures
            .iter()
            .filter(|r| r.scope == mine.uid)
            .cloned()
            .collect()
    };
    let (now_rows, was_rows) = (rows(&f), rows(&drive_file));
    if let Some(gone) = was_rows.iter().find(|r| !now_rows.contains(r)) {
        return Err(format!(
            "it leaves out a signature by {} that the drive's file holds: a signature is never taken away",
            gone.signer
        ));
    }
    if let Some(theirs) = now_rows
        .iter()
        .find(|r| !was_rows.contains(r) && r.signer != me)
    {
        return Err(format!(
            "it brings a signature by {}: only {me}'s own goes back with {me}'s work",
            theirs.signer
        ));
    }
    // The requests: every one the drive's file holds is kept as it is, with
    // its signatures, and the only ones added are `me`'s own, asked and
    // signed by them (F3-d).
    let asked = requests_verdict(&f, &drive_file, &mine.uid, &mine.id, me, at, registry)?;
    // Everything else outside the node's rows is as the drive holds it.
    let unsigned = |x: &File| {
        let mut x = x.clone();
        x.signatures.retain(|r| {
            r.scope != mine.uid && !asked.iter().any(|u| r.scope == chain::request_scope(u))
        });
        x.requests.retain(|r| !asked.contains(&r.uid));
        x
    };
    let changed = save::changed_outside(&unsigned(&f), &unsigned(&drive_file), &mine.uid);
    if !changed.is_empty() {
        return Err(format!(
            "it changes what is not {}'s own to change ({}): a node's file changes the node's own rows, and nothing else",
            mine.id,
            changed.join(", ")
        ));
    }
    let revision = save::revision_of(&f, path).map_err(said)?;
    if mine.revision != revision as i64 {
        return Err(format!(
            "it says it is revision {revision}, and its block says {}: it is one revision",
            mine.revision
        ));
    }
    let drive_revision = save::revision_of(&drive_file, path).map_err(said)?;
    if revision < drive_revision {
        return Err(format!(
            "it is revision {revision}, and the drive holds revision {drive_revision}: a node's file goes back as a later revision, or as the same one signed"
        ));
    }
    if revision == drive_revision {
        // The same revision: the drive's file, and nothing more than `me`'s
        // signature and the program that wrote it.
        let bare = |x: &File| {
            let mut x = unsigned(x);
            x.meta.remove("written_by_app");
            x
        };
        if bare(&f) != bare(&drive_file) {
            return Err(format!(
                "it is revision {revision}, as the drive's file is, and differs from it in more than {me}'s signature and requests: a changed node goes back as a later revision"
            ));
        }
        // Only asked: the node is the drive's own, as its signatures are, so
        // nothing of it is `me`'s to sign. The requests go back by themselves.
        if !asked.is_empty() && now_rows == was_rows {
            return Ok(format!(
                "goes back: {} on the node {}, asked and signed by {me}",
                if asked.len() == 1 {
                    "a request".to_string()
                } else {
                    format!("{} requests", asked.len())
                },
                mine.id
            ));
        }
    }
    let writer = f.meta.get("writer").map_or("", String::as_str);
    if !registry.may_sign_node(writer, &mine.uid) {
        return Err(format!(
            "it says {writer:?} wrote it, who may not write {} in {}",
            mine.id, registry.from
        ));
    }
    // One signature by `me` both counts as it was made and is made with a key
    // that still counts now: the chain is read on only those. Why not is said
    // as which of the two fails.
    if let Err(now) = chain::check_node(
        &only_counting_now(&f, &mine.uid, me, at, registry),
        &mine.uid,
        registry,
    ) {
        // Requests go back by themselves only on the drive's own revision:
        // on a later one they go back with it, signed.
        let requests = if asked.is_empty() {
            String::new()
        } else {
            format!(
                " Its requests go back by themselves only while the node is the drive's revision {drive_revision}: sign revision {revision}, and they go back with it"
            )
        };
        return Err(
            match chain::check_node(&only_by(&f, &mine.uid, me), &mine.uid, registry) {
                Err(e) => format!(
                    "it carries no current signature by {me} that counts on the drive: {}.{requests}",
                    e.message()
                ),
                Ok(()) => format!(
                    "the key {me} signed it with does not count for them at {at}, when it would go back: {}",
                    now.message()
                ),
            },
        );
    }
    let found = checks::check_node(&f);
    let refusing: Vec<String> = found
        .errors()
        .map(|e| format!("{}: {}", e.place, e.what))
        .collect();
    if !refusing.is_empty() {
        return Err(format!(
            "its checks refuse it ({}) — {}",
            refusing.len(),
            refusing.join("; ")
        ));
    }
    Ok(format!("goes back: the node {}, signed by {me}", mine.id))
}

/// The requests `f` adds to `drive_file`'s on the node `uid` (`id`), when each
/// is `me`'s own, open, unanswered and signed by them with a key that counts
/// now; or why not. Every request the drive's file holds must be in `f` as it
/// is, and every signature of one: a request is kept as it was asked.
fn requests_verdict(
    f: &File,
    drive_file: &File,
    uid: &str,
    id: &str,
    me: &str,
    at: &str,
    registry: &Registry,
) -> Result<Vec<String>, String> {
    // A uid is a request's whole name: two rows that share one are refused,
    // never read as the first of them.
    for (k, r) in f.requests.iter().enumerate() {
        if f.requests[..k].iter().any(|o| o.uid == r.uid) {
            return Err(format!(
                "it holds more than one request {:?}: a request's uid is its own",
                r.uid
            ));
        }
    }
    for r in &drive_file.requests {
        if !f.requests.contains(r) {
            return Err(format!(
                "it changes or leaves out the request {} by {} that the drive's file holds: a request is kept as it was asked",
                r.uid, r.raised_by
            ));
        }
        let scope = chain::request_scope(&r.uid);
        if let Some(gone) = drive_file
            .signatures
            .iter()
            .find(|s| s.scope == scope && !f.signatures.contains(s))
        {
            return Err(format!(
                "it leaves out {}'s signature of the request {}: a signature is never taken away",
                gone.signer, r.uid
            ));
        }
        if let Some(added) = f
            .signatures
            .iter()
            .find(|s| s.scope == scope && !drive_file.signatures.contains(s))
        {
            return Err(format!(
                "it brings a signature by {} of the request {}, which the drive's file holds already: a request is signed once, when it is asked",
                added.signer, r.uid
            ));
        }
    }
    let mut asked = Vec::new();
    for r in f
        .requests
        .iter()
        .filter(|r| !drive_file.requests.iter().any(|d| d.uid == r.uid))
    {
        if r.scope != uid {
            return Err(format!(
                "it brings the request {}, on {:?}: a node's file asks of its own node",
                r.uid, r.scope
            ));
        }
        if r.raised_by != me {
            return Err(format!(
                "it brings the request {} by {}: only {me}'s own goes back with {me}'s work",
                r.uid, r.raised_by
            ));
        }
        if r.status != "open" || !r.answer.is_empty() {
            return Err(format!(
                "it brings the request {} already answered ({}): a request is answered by its group, not by whoever asked",
                r.uid, r.status
            ));
        }
        let scope = chain::request_scope(&r.uid);
        if let Some(old) = drive_file.signatures.iter().find(|s| s.scope == scope) {
            return Err(format!(
                "it brings the request {}, and the drive's file holds a signature by {} of a request of that name it does not hold: a signature is never taken away",
                r.uid, old.signer
            ));
        }
        if let Some(theirs) = f
            .signatures
            .iter()
            .find(|s| s.scope == scope && s.signer != me)
        {
            return Err(format!(
                "it brings a signature by {} of {me}'s request {}: only {me}'s own goes back with {me}'s work",
                theirs.signer, r.uid
            ));
        }
        if let Err(now) = chain::check_request(
            &only_counting_now(f, &scope, me, at, registry),
            &r.uid,
            registry,
        ) {
            return Err(
                match chain::check_request(&only_by(f, &scope, me), &r.uid, registry) {
                    Err(e) => format!(
                        "its request {} on {id} does not count on the drive: {}",
                        r.uid,
                        e.message()
                    ),
                    Ok(()) => format!(
                        "the key {me} signed the request {} with does not count for them at {at}, when it would go back: {}",
                        r.uid,
                        now.message()
                    ),
                },
            );
        }
        asked.push(r.uid.clone());
    }
    Ok(asked)
}

/// Whether the changed file at `path` goes back as `me`'s work: why it does,
/// or why not.
fn verdict(
    path: &str,
    bytes: &[u8],
    theirs: Option<&[u8]>,
    me: &str,
    at: &str,
    chain: &Chain,
    drive: &Path,
) -> Result<String, String> {
    let parts: Vec<&str> = path.split('/').collect();
    match parts.as_slice() {
        ["groups", g, "nodes", name] if name.ends_with(".vnode") => {
            node_verdict(path, g, bytes, theirs, me, at, chain)
        }
        ["groups", g, "releases", name] if name.ends_with(".vleo") => {
            release_verdict(g, name, bytes, theirs, me, at, chain, drive)
        }
        ["design", name] if name.starts_with("design-") && name.ends_with(".vleo") => {
            design_verdict(name, bytes, theirs, me, at, chain, drive)
        }
        ["design", "NOTES.md"] | ["readable", _] => record_verdict(path, me, chain),
        ["groups", g, name] if *name == format!("{g}.vgroup") => Err(format!(
            "{g}'s group file does not go back yet: what may change in a group's file is not yet checked against the drive's, so a change of its people stays in your copy"
        )),
        _ => Err("it is not a file the application puts back yet: a node's file, a group's release, or a released design and its record, goes back".into()),
    }
}

/// Whether the release `name` in `group`'s `releases/`, `bytes` in the copy,
/// goes back as a new file sealed by `me`, onto the drive at `drive` that
/// holds `theirs` at that path.
#[allow(clippy::too_many_arguments)]
fn release_verdict(
    group: &str,
    name: &str,
    bytes: &[u8],
    theirs: Option<&[u8]>,
    me: &str,
    at: &str,
    chain: &Chain,
    drive: &Path,
) -> Result<String, String> {
    if theirs.is_some() {
        return Err(
            "the drive holds a release at this path already: a release is never written over"
                .into(),
        );
    }
    let f = save::parsed(bytes).map_err(|e| format!("it does not read: {}", e.message()))?;
    match f.kind() {
        Ok(Kind::GroupRelease) => {}
        Ok(other) => {
            return Err(format!(
                "it is a {other}, and releases/ holds releases only"
            ))
        }
        Err(e) => return Err(e.message().to_string()),
    }
    // A release sealed in a copy is the one schema's: one upgraded from
    // before 1.0 is checked on the programme manager's anchor alone, never
    // on a seal by the person putting it back.
    if f.meta.contains_key("upgraded_from") {
        return Err(
            "it is a release from before 1.0, upgraded: a release goes back only as sealed in the application"
                .into(),
        );
    }
    f.check_meta().map_err(|e| e.message().to_string())?;
    f.check_values().map_err(|e| e.message().to_string())?;
    let said = |k: &str| f.meta.get(k).map_or("", String::as_str);
    if said("group_id") != group {
        return Err(format!(
            "it says it is {:?}'s release, in {group}'s folder",
            said("group_id")
        ));
    }
    let version = said("version");
    let Some(v) = crate::meta::ReleaseVersion::parse(version) else {
        return Err(format!(
            "it says its version is {version:?}, which is not a release's: major.minor"
        ));
    };
    if name != format!("{group}-{version}.vleo") {
        return Err(format!(
            "it says it is {group} {version}, so its name is {group}-{version}.vleo"
        ));
    }
    if said("sealed_by") != me {
        return Err(format!(
            "it is sealed by {:?}: a release goes back only by the person who sealed it",
            said("sealed_by")
        ));
    }
    // After the drive's latest release, as the drive holds it now: one sealed
    // after another that is not the drive's latest was sealed beside a
    // release nobody it followed knew of.
    let latest = release::releases_in(
        &drive.join("groups").join(group).join("releases"),
        group,
        concat!("vleo ", env!("CARGO_PKG_VERSION")),
    )
    .map_err(|e| {
        format!(
            "the drive's releases of {group} do not read: {}",
            e.message()
        )
    })?
    .into_iter()
    .next();
    if let Some((l, _)) = &latest {
        if v <= *l {
            return Err(format!(
                "it is {group} {version}, and the drive's latest release of {group} is {l}: a release comes after the latest"
            ));
        }
    }
    let latest = latest.map(|(l, _)| l.to_string());
    if said("previous") != latest.as_deref().unwrap_or("") {
        return Err(format!(
            "it follows {}, and the drive's latest release of {group} is {}: seal it again on a copy taken now",
            if said("previous").is_empty() { "no release" } else { said("previous") },
            latest.as_deref().unwrap_or("none")
        ));
    }
    let systems = chain.systems.as_ref().ok_or_else(|| {
        "the systems' file on the drive does not check through the chain, so no seal counts"
            .to_string()
    })?;
    let (parent, _) = chain::mounted_by(group, &[&chain.programme, systems])
        .map_err(|e| e.message().to_string())?;
    // Its nodes are counted through the group file it carries; that file's
    // people, keys and assignments must be the drive's group file's as it
    // is now, or a copy taken before a change, or one with a person invented
    // in it, would carry them onto the drive inside the release.
    let theirs_group = registry_of(chain, group)?;
    for (table, same) in [
        ("people", same_rows(&f.people, &theirs_group.file.people)),
        ("keys", same_rows(&f.keys, &theirs_group.file.keys)),
        (
            "assignments",
            same_rows(&f.assignments, &theirs_group.file.assignments),
        ),
    ] {
        if !same {
            return Err(format!(
                "its {table} are not the drive's group file's as it is now: a release is sealed on a copy whose group file is the drive's; copy the drive again, and seal it there"
            ));
        }
    }
    let found = checks::check_release(&f);
    let refusing: Vec<String> = found
        .errors()
        .map(|e| format!("{}: {}", e.place, e.what))
        .collect();
    if !refusing.is_empty() {
        return Err(format!("its checks refuse it — {}", refusing.join("; ")));
    }
    // The seal by `me` with a key the drive's programme's file counts for
    // them now, and every node it holds, checked through the chain as the
    // drive holds it.
    let now = only_counting_now(&f, chain::FILE, me, at, &chain.programme);
    let checked = chain::check_release_in(&now, parent, &chain.programme)
        .map_err(|e| e.message().to_string())?;
    if !checked.holds() {
        return Err(format!(
            "it does not check through the chain as the drive holds it now, sealed by {me} with a key that counts at {at}: {}",
            checked.refused.join("; ")
        ));
    }
    Ok(format!(
        "goes back: {group} {version}, sealed by {me}, a new release"
    ))
}

/// Whether the released design `name` in `design/`, `bytes` in the copy,
/// goes back as a new file released by `me`, onto the drive at `drive` that
/// holds `theirs` at that path (F6-b).
fn design_verdict(
    name: &str,
    bytes: &[u8],
    theirs: Option<&[u8]>,
    me: &str,
    at: &str,
    chain: &Chain,
    drive: &Path,
) -> Result<String, String> {
    if theirs.is_some() {
        return Err(
            "the drive holds a released design at this path already: a release is never written over"
                .into(),
        );
    }
    let f = save::parsed(bytes).map_err(|e| format!("it does not read: {}", e.message()))?;
    match f.kind() {
        Ok(Kind::Design) => {}
        Ok(other) => {
            return Err(format!(
                "it is a {other}, and design/ holds released designs only"
            ))
        }
        Err(e) => return Err(e.message().to_string()),
    }
    let said = |k: &str| f.meta.get(k).map_or("", String::as_str);
    let version = said("version");
    if name != format!("design-{version}.vleo") {
        return Err(format!(
            "it says it is design {version}, so its name is design-{version}.vleo"
        ));
    }
    if said("released_by") != me {
        return Err(format!(
            "it is released by {:?}: a released design goes back only by the person who released it",
            said("released_by")
        ));
    }
    // After the drive's latest released design, as the drive holds it now.
    f.check_meta().map_err(|e| e.message().to_string())?;
    let latest = crate::design::latest_in(&drive.join("design"), &chain.programme)
        .map_err(|e| format!("the drive's released designs do not read: {}", e.message()))?;
    if said("previous") != latest.as_deref().unwrap_or("") {
        return Err(format!(
            "it follows {}, and the drive's latest released design is {}: release it again on a copy taken now",
            if said("previous").is_empty() { "no released design" } else { said("previous") },
            latest.as_deref().unwrap_or("none")
        ));
    }
    crate::design::follows(version, latest.as_deref())?;
    let now = only_counting_now(&f, chain::FILE, me, at, &chain.programme);
    chain::check_design(&now, &chain.programme).map_err(|e| {
        format!(
            "it does not check through the chain as the drive holds it now, released by {me} with a key that counts at {at}: {}",
            e.message()
        )
    })?;
    Ok(format!(
        "goes back: design {version}, released by {me}, a new released design"
    ))
}

/// Whether `path`, the record of the released design (`design/NOTES.md`, a
/// file of `readable/`), goes back as `me`'s: the system engineer's, or their
/// deputy's, as the drive's programme's file counts them, over the drive's
/// file only when it is as copied (question 38).
fn record_verdict(path: &str, me: &str, chain: &Chain) -> Result<String, String> {
    if path.starts_with("readable/") && !path.ends_with(".csv") {
        return Err("readable/ holds CSV only".into());
    }
    if !chain.programme.acts_as(me, "system engineer") {
        return Err(format!(
            "{me} is not the system engineer, nor their deputy: the released design's record is theirs to write"
        ));
    }
    Ok(format!(
        "goes back: {path}, the released design's record, by {me}"
    ))
}

/// Whether `a` and `b` hold the same rows, in whatever order.
fn same_rows<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    a.len() == b.len() && a.iter().all(|x| b.contains(x)) && b.iter().all(|x| a.contains(x))
}

/// The report of what in the working copy at `copy` would go back to the
/// drive at `drive` as `me`'s work at the moment `at`, checked through the
/// chain `open` opens from the drive's own files. With `write` naming the
/// files, each with the digest the report gave it, and those exactly the
/// ones that go back as the report is made again under the drive's lock,
/// they are written on the drive, and the copy's record says so; otherwise
/// nothing is written, and a list that differs is refused with the report's
/// own.
pub fn put_back(
    copy: &Path,
    drive: &Path,
    me: &str,
    at: &str,
    open: &dyn Fn() -> Result<Chain, Error>,
    write: Option<&[(String, String)]>,
) -> Result<Report, Error> {
    let _held = save::lock(copy)?;
    let mut record = record_of(copy)?;
    let drive_root = std::fs::canonicalize(drive).map_err(|e| io(drive, e))?;
    if Path::new(&record.drive) != drive_root {
        return Err(refused(format!(
            "this copy came from {}, not {}: work goes back only to the drive it came from",
            record.drive,
            drive_root.display()
        )));
    }
    // To write, the drive's lock is taken first, and the chain is opened under
    // it: what goes back is judged against the drive as it is when it goes.
    let _drive_held = match write {
        Some(_) => Some(drive_lock(&drive_root)?),
        None => None,
    };
    let chain = open()?;
    let now = digests(copy)?;
    let mut report = Report::default();
    for path in record.changed(&now) {
        let line = |back: bool, why: String, digest: &str| Line {
            path: path.clone(),
            back,
            why,
            digest: digest.to_string(),
        };
        let Some(mine) = now.get(&path) else {
            report.lines.push(line(
                false,
                "gone from your copy: nothing is deleted on the drive".into(),
                "",
            ));
            continue;
        };
        let there = match on_drive(&drive_root, &path) {
            Ok(t) => t,
            Err(e) => {
                report
                    .lines
                    .push(line(false, e.message().to_string(), mine));
                continue;
            }
        };
        let there_digest = there.as_deref().map(copied::digest);
        if there_digest.as_ref() == Some(mine) {
            report.lines.push(line(false, ALREADY.into(), mine));
            continue;
        }
        if there_digest.as_ref() != record.files.get(&path) {
            report.lines.push(line(
                false,
                format!(
                    "changed on the drive since your copy was taken, so it is never written over: keep yours aside, copy the database again, and make your change on what is there now ({})",
                    if there.is_none() { "it is gone from the drive" } else { "someone else wrote it" }
                ),
                mine,
            ));
            continue;
        }
        let at_copy = copy.join(&path);
        let bytes = std::fs::read(&at_copy).map_err(|e| io(&at_copy, e))?;
        if copied::digest(&bytes) != *mine {
            return Err(refused(format!(
                "{path} changed in your copy while it was being read: look again"
            )));
        }
        match verdict(&path, &bytes, there.as_deref(), me, at, &chain, &drive_root) {
            Ok(why) => report.lines.push(line(true, why, mine)),
            Err(why) => report.lines.push(line(false, why, mine)),
        }
    }
    one_design_at_a_time(&mut report);
    let Some(named) = write else {
        return Ok(report);
    };
    let mut named: Vec<(String, String)> = named.to_vec();
    named.sort();
    named.dedup();
    let back = report.named();
    let already: Vec<(String, String)> = report
        .lines
        .iter()
        .filter(|l| l.why == ALREADY)
        .map(|l| (l.path.clone(), l.digest.clone()))
        .collect();
    if back.is_empty() && already.is_empty() {
        return Err(refused("nothing in your copy goes back to the drive now"));
    }
    if named != back {
        return Err(refused(format!(
            "the files named are not the ones that go back now, as your copy holds them ({}), so nothing is written: look at the report again",
            if back.is_empty() {
                "none".to_string()
            } else {
                back.iter().map(|(p, _)| p.as_str()).collect::<Vec<_>>().join(", ")
            }
        )));
    }
    // Once more, just before anything is written: each file on the drive as
    // the record holds it, and in the copy as the report gave it.
    let mut bytes_of = Vec::new();
    for (path, digest) in &back {
        let there = on_drive(&drive_root, path)?.map(|b| copied::digest(&b));
        if there.as_ref() != record.files.get(path) {
            return Err(refused(format!(
                "{path} changed on the drive while it was being put back, so nothing is written: look at the report again"
            )));
        }
        let at_copy = copy.join(path);
        let bytes = std::fs::read(&at_copy).map_err(|e| io(&at_copy, e))?;
        if copied::digest(&bytes) != *digest {
            return Err(refused(format!(
                "{path} changed in your copy while it was being put back, so nothing is written: look at the report again"
            )));
        }
        bytes_of.push((path.clone(), bytes));
    }
    let mut done = Vec::new();
    let mut failed = None;
    for (n, (path, bytes)) in bytes_of.iter().enumerate() {
        let new = !record.files.contains_key(path);
        match write_on_drive(&drive_root, path, bytes, n, new) {
            Ok(()) => {
                record.files.insert(path.clone(), copied::digest(bytes));
                done.push(path.clone());
            }
            Err(e) => {
                failed = Some((path.clone(), e));
                break;
            }
        }
    }
    // A file already on the drive as the copy holds it is the drive's now too.
    for (path, digest) in &already {
        record.files.insert(path.clone(), digest.clone());
    }
    let went = if done.is_empty() {
        "Nothing".to_string()
    } else {
        done.join(", ")
    };
    let failed_too = failed.as_ref().map_or(String::new(), |(path, e)| {
        format!(
            " {path} could not be written on the drive ({}), and the files after it were not",
            e.message()
        )
    });
    if let Err(e) = write_record(copy, &record) {
        return Err(Error::new(
            ErrorKind::Io,
            format!(
                "{went} went back to the drive, but your copy's record could not say so: {}.{failed_too} Copy the database again only once this is put right",
                e.message()
            ),
        ));
    }
    if let Some((path, e)) = failed {
        return Err(Error::new(
            ErrorKind::Io,
            format!(
                "{path} could not be written on the drive: {}. {went} went back before it, and the rest did not; nothing someone else changed was written over",
                e.message()
            ),
        ));
    }
    report.recorded = already.iter().map(|(p, _)| p.clone()).collect();
    report.written = !done.is_empty();
    Ok(report)
}

/// Whether `path` is a released design in `design/`.
fn is_design(path: &str) -> bool {
    path.strip_prefix("design/")
        .is_some_and(|n| n.starts_with("design-") && n.ends_with(".vleo") && !n.contains('/'))
}

/// Whether `path` is the record of a released design: `design/NOTES.md`, a
/// file of `readable/`.
fn is_record(path: &str) -> bool {
    path == "design/NOTES.md" || path.starts_with("readable/")
}

/// One released design goes back at a time, each after the drive's latest,
/// so two never follow the same one; and its record goes back only beside
/// it, so a record never says of the drive what no released design there
/// says. Said as why, never dropped.
fn one_design_at_a_time(report: &mut Report) {
    let designs = report
        .lines
        .iter()
        .filter(|l| l.back && is_design(&l.path))
        .count();
    for l in report.lines.iter_mut().filter(|l| l.back) {
        if designs > 1 && is_design(&l.path) {
            l.back = false;
            l.why = format!(
                "{designs} released designs would go back at once, each after the drive's latest: one goes back at a time, then the next is released on a copy taken after it"
            );
        } else if designs != 1 && is_record(&l.path) {
            l.back = false;
            l.why = "the released design's record goes back only beside the released design it is of, put back at the same time".into();
        }
    }
}

/// `bytes` written whole at `path` on the drive: to a name of the program's
/// own beside it, then renamed over it, so a write cut short leaves the
/// drive's file as it was. Never through a link, and never into a folder the
/// drive does not hold.
fn write_on_drive(
    drive: &Path,
    path: &str,
    bytes: &[u8],
    n: usize,
    new: bool,
) -> Result<(), Error> {
    use std::io::Write as _;
    no_link_on_the_way(drive, path)?;
    let to = drive.join(path);
    let dir = to
        .parent()
        .ok_or_else(|| refused(format!("{path} has no folder on the drive")))?;
    // A group's first release makes its `releases/`, in the group's folder
    // the drive holds, and the first released design `design/` and
    // `readable/` at the top; no other folder is made.
    let top = matches!(
        path.split('/').collect::<Vec<_>>().as_slice(),
        ["design" | "readable", _]
    );
    if (top || dir.file_name().is_some_and(|n| n == "releases")) && !dir.exists() {
        std::fs::create_dir(dir).map_err(|e| io(dir, e))?;
        no_link_on_the_way(drive, path)?;
    }
    let name = to
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let tmp = dir.join(format!("{INCOMING}{}-{n}-{name}", token()));
    let made = (|| -> Result<(), Error> {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(|e| io(&tmp, e))?;
        f.write_all(bytes).map_err(|e| io(&tmp, e))?;
        f.sync_all().map_err(|e| io(&tmp, e))?;
        drop(f);
        no_link_on_the_way(drive, path)?;
        if new {
            // A new file is linked into its place, which fails when anything
            // is there, whoever wrote it since: it is never written over.
            std::fs::hard_link(&tmp, &to).map_err(|e| io(&to, e))?;
            let _ = std::fs::remove_file(&tmp);
            Ok(())
        } else {
            std::fs::rename(&tmp, &to).map_err(|e| io(&to, e))
        }
    })();
    if made.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    made
}

/// The copy's record written whole: beside itself first, under a name of the
/// program's own, then renamed.
fn write_record(copy: &Path, record: &Copied) -> Result<(), Error> {
    let at = copy.join(copied::FILE_NAME);
    let tmp = copy.join(format!("{RECORD}{}", token()));
    std::fs::write(&tmp, record.to_toml()).map_err(|e| io(&tmp, e))?;
    std::fs::rename(&tmp, &at).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        io(&at, e)
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_written_through_a_link_even_when_one_appears_after_the_report() {
        let at = std::env::temp_dir().join(format!("vleo-write-link-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        let (drive, outside) = (at.join("drive"), at.join("outside"));
        std::fs::create_dir_all(drive.join("groups/g")).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("f.vnode"), "theirs").unwrap();
        std::os::unix::fs::symlink(&outside, drive.join("groups/g/nodes")).unwrap();
        let e = write_on_drive(&drive, "groups/g/nodes/f.vnode", b"mine", 0, false).unwrap_err();
        assert!(
            e.message().contains("reached through a link"),
            "{}",
            e.message()
        );
        assert_eq!(std::fs::read(outside.join("f.vnode")).unwrap(), b"theirs");
        assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 1);
        let _ = std::fs::remove_dir_all(&at);
    }
}
