//! A person's own work put back on the shared drive from their working copy
//! (docs/briefs/F3.md, F3-c; docs/PLAN_F.md, decision 3).
//!
//! The copy's record (`copied.toml`) says what every file was when the copy
//! was taken. A file whose bytes in the copy are not that is the person's own
//! work, and it goes back only when all of these hold:
//!
//! - it differs from the drive;
//! - the drive's bytes are still the ones the record holds: a file someone
//!   else changed on the drive since the copy is never written over;
//! - it carries a current signature, or seal, by the person, which counts
//!   through the chain as the **drive** holds it now: a key revoked there
//!   since the copy is seen;
//! - it is a kind of file the application puts back: a node's file, whose
//!   own checks hold, or a group's file the person changed at their valve.
//!
//! Every other changed file is named, with the reason. Nothing is deleted on
//! the drive, and a file gone from the copy is said, not followed.
//!
//! [`put_back`] answers the report and writes nothing, until it is given the
//! files to put back, and those are exactly the ones the report says go back
//! as it is made again. Each file is written to a temporary name beside its
//! place on the drive, then renamed over it, while the drive's lock is held;
//! the copy's record then says the file is as the drive holds it.
//!
//! The locks are of files: the copy's beside it ([`crate::save::lock`]), the
//! drive's in its own folder. Two programs that put back through this
//! library never write one file at once; a sync client or a person who
//! writes the drive by hand is not held by them.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::chain::{self, Registry, FILE};
use crate::checks;
use crate::copied::{self, Copied};
use crate::error::{Error, ErrorKind};
use crate::meta::Kind;
use crate::model::File;

/// The drive's chain as the caller opened it, from the drive's own files:
/// the anchor START HERE names, the programme's file it opened, the systems'
/// file and every group's file that opened through it, and why each other
/// was left out.
pub struct Drive<'a> {
    pub anchor: &'a str,
    pub programme: &'a Registry,
    pub systems: Option<&'a Registry>,
    pub groups: &'a [Registry],
    pub left_out: &'a [String],
}

/// One changed file: whether it goes back, and why, or why not.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub path: String,
    pub back: bool,
    pub why: String,
}

/// What a put-back found, file by file, and whether it wrote.
#[derive(Debug, Default)]
pub struct Report {
    pub lines: Vec<Line>,
    pub written: bool,
}

impl Report {
    /// The paths that go back, in order.
    pub fn back(&self) -> Vec<String> {
        self.lines
            .iter()
            .filter(|l| l.back)
            .map(|l| l.path.clone())
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
/// program writes the drive, and let go when dropped.
struct DriveLock(PathBuf);

impl Drop for DriveLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// The name the drive's lock and every file being written there begin with:
/// the program's own, never a file of the database's.
const OWN: &str = ".vleo-";

fn drive_lock(drive: &Path) -> Result<DriveLock, Error> {
    let at = drive.join(format!("{OWN}put-back.lock"));
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&at)
    {
        Ok(mut f) => {
            use std::io::Write as _;
            let _ = writeln!(f, "{}", std::process::id());
            Ok(DriveLock(at))
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
/// and the SHA-256 of its bytes. The record itself, and the program's own
/// lock and files being written on a drive (`.vleo-…`), are not among them.
/// A link, a key, anything but a folder or a file, or a name the record could
/// not hold is refused by name, never followed or skipped.
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
            if shown == copied::FILE_NAME || name.starts_with(OWN) {
                continue;
            }
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

/// The bytes of the file `path` names on the drive, if there is one: an
/// ordinary file, never a link. `None` when the drive holds no such file.
fn on_drive(drive: &Path, path: &str) -> Result<Option<Vec<u8>>, Error> {
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

/// Whether the changed file at `path` goes back as `me`'s work: why it does,
/// or why not. `bytes` are the file's bytes in the copy.
fn verdict(path: &str, bytes: &[u8], me: &str, drive: &Drive) -> Result<String, String> {
    let parts: Vec<&str> = path.split('/').collect();
    let read =
        || crate::save::parsed(bytes).map_err(|e| format!("it does not read: {}", e.message()));
    match parts.as_slice() {
        ["groups", g, "nodes", name] if name.ends_with(".vnode") => {
            let f = read()?;
            crate::save::the_node(&f, path).map_err(|e| e.message().to_string())?;
            let uid = f.blocks[0].uid.clone();
            let group = drive
                .groups
                .iter()
                .find(|r| r.file.meta.get("group_id").map(String::as_str) == Some(*g))
                .ok_or_else(|| {
                    let why = drive
                        .left_out
                        .iter()
                        .find(|w| w.starts_with(&format!("{g}: ")))
                        .map_or(String::new(), |w| format!(": {w}"));
                    format!("{g}'s group file on the drive does not check through the chain, so no signature in it counts{why}")
                })?;
            chain::check_node(&only_by(&f, &uid, me), &uid, group).map_err(|e| {
                format!(
                    "it carries no current signature by {me} that counts on the drive: {}",
                    e.message()
                )
            })?;
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
            Ok(format!("goes back: the node {}, signed by {me}", f.blocks[0].id))
        }
        ["groups", g, name] if *name == format!("{g}.vgroup") => {
            let f = read()?;
            if f.kind().map_err(|e| e.message().to_string())? != Kind::Group
                || f.meta.get("group_id").map(String::as_str) != Some(*g)
            {
                return Err(format!(
                    "it is not {g}'s group file: a group's file lives in its own group's folder"
                ));
            }
            let mine = only_by(&f, FILE, me);
            let opened = match *g {
                "programme" => chain::open_programme(mine, drive.anchor).map(|_| ()),
                "systems" => chain::open_systems(&mine, drive.programme).map(|_| ()),
                _ => {
                    let parents: Vec<&Registry> = std::iter::once(drive.programme)
                        .chain(drive.systems)
                        .collect();
                    chain::mounted_by(g, &parents).and_then(|(parent, _)| {
                        chain::open_group_in(&mine, parent, drive.programme).map(|_| ())
                    })
                }
            };
            opened.map_err(|e| {
                format!(
                    "it carries no current signature by {me} that counts on the drive: {}",
                    e.message()
                )
            })?;
            Ok(format!("goes back: {g}'s group file, signed by {me}"))
        }
        _ => Err(
            "it is not a file the application puts back yet: a node's file, or a group's file changed at its valve"
                .into(),
        ),
    }
}

/// The report of what in the working copy at `copy` would go back to the
/// drive at `drive` as `me`'s work, checked through `chain`, the drive's own.
/// With `write` naming the files, and those exactly the ones that go back as
/// the report is made now, they are written on the drive, and the copy's
/// record says so; otherwise nothing is written, and a list that differs is
/// refused with the report's own.
pub fn put_back(
    copy: &Path,
    drive: &Path,
    me: &str,
    chain: &Drive,
    write: Option<&[String]>,
) -> Result<Report, Error> {
    let _held = crate::save::lock(copy)?;
    let mut record = record_of(copy)?;
    let drive_root = std::fs::canonicalize(drive).map_err(|e| io(drive, e))?;
    if Path::new(&record.drive) != drive_root {
        return Err(refused(format!(
            "this copy came from {}, not {}: work goes back only to the drive it came from",
            record.drive,
            drive_root.display()
        )));
    }
    let now = digests(copy)?;
    let mut report = Report::default();
    for path in record.changed(&now) {
        let line = |back: bool, why: String| Line {
            path: path.clone(),
            back,
            why,
        };
        let Some(mine) = now.get(&path) else {
            report.lines.push(line(
                false,
                "gone from your copy: nothing is deleted on the drive".into(),
            ));
            continue;
        };
        let there = on_drive(&drive_root, &path)?;
        let there_digest = there.as_deref().map(copied::digest);
        if there_digest.as_ref() == Some(mine) {
            report.lines.push(line(
                false,
                "already on the drive as it is in your copy".into(),
            ));
            continue;
        }
        if there_digest.as_ref() != record.files.get(&path) {
            report.lines.push(line(
                false,
                format!(
                    "changed on the drive since your copy was taken, so it is never written over: keep yours aside, copy the database again, and make your change on what is there now ({})",
                    if there.is_none() { "it is gone from the drive" } else { "someone else wrote it" }
                ),
            ));
            continue;
        }
        let at = copy.join(&path);
        let bytes = std::fs::read(&at).map_err(|e| io(&at, e))?;
        if copied::digest(&bytes) != *mine {
            return Err(refused(format!(
                "{path} changed in your copy while it was being read: look again"
            )));
        }
        match verdict(&path, &bytes, me, chain) {
            Ok(why) => report.lines.push(line(true, why)),
            Err(why) => report.lines.push(line(false, why)),
        }
    }
    let Some(named) = write else {
        return Ok(report);
    };
    let mut named: Vec<String> = named.to_vec();
    named.sort();
    named.dedup();
    let back = report.back();
    if back.is_empty() {
        return Err(refused("nothing in your copy goes back to the drive now"));
    }
    if named != back {
        return Err(refused(format!(
            "the files named are not the ones that go back now ({}), so nothing is written: look at the report again",
            back.join(", ")
        )));
    }
    let _drive_held = drive_lock(&drive_root)?;
    // Under the drive's lock, each file once more as the record holds it, and
    // as the copy held it when it was checked, before anything is written.
    let mut bytes_of = Vec::new();
    for path in &back {
        let there = on_drive(&drive_root, path)?.map(|b| copied::digest(&b));
        if there.as_ref() != record.files.get(path) {
            return Err(refused(format!(
                "{path} changed on the drive while it was being put back, so nothing is written: look at the report again"
            )));
        }
        let at = copy.join(path);
        let bytes = std::fs::read(&at).map_err(|e| io(&at, e))?;
        if Some(&copied::digest(&bytes)) != now.get(path) {
            return Err(refused(format!(
                "{path} changed in your copy while it was being put back, so nothing is written: look at the report again"
            )));
        }
        bytes_of.push((path.clone(), bytes));
    }
    let mut done = Vec::new();
    let mut failed = None;
    for (n, (path, bytes)) in bytes_of.iter().enumerate() {
        match write_on_drive(&drive_root, path, bytes, n) {
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
    // The record says what went back, whatever did not.
    write_record(copy, &record)?;
    if let Some((path, e)) = failed {
        return Err(Error::new(
            ErrorKind::Io,
            format!(
                "{path} could not be written on the drive: {}. {} went back before it; the rest did not, and nothing on the drive was written over",
                e.message(),
                if done.is_empty() { "Nothing".to_string() } else { done.join(", ") }
            ),
        ));
    }
    report.written = true;
    Ok(report)
}

/// `bytes` written whole at `path` on the drive: to a name of the program's
/// own beside it, then renamed over it, so a write cut short leaves the
/// drive's file as it was.
fn write_on_drive(drive: &Path, path: &str, bytes: &[u8], n: usize) -> Result<(), Error> {
    use std::io::Write as _;
    let to = drive.join(path);
    let dir = to
        .parent()
        .ok_or_else(|| refused(format!("{path} has no folder on the drive")))?;
    std::fs::create_dir_all(dir).map_err(|e| io(dir, e))?;
    let name = to
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let tmp = dir.join(format!("{OWN}incoming-{}-{n}-{name}", std::process::id()));
    let made = (|| -> Result<(), Error> {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(|e| io(&tmp, e))?;
        f.write_all(bytes).map_err(|e| io(&tmp, e))?;
        f.sync_all().map_err(|e| io(&tmp, e))?;
        drop(f);
        std::fs::rename(&tmp, &to).map_err(|e| io(&to, e))
    })();
    if made.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    made
}

/// The copy's record written whole: beside itself first, then renamed.
fn write_record(copy: &Path, record: &Copied) -> Result<(), Error> {
    let at = copy.join(copied::FILE_NAME);
    let tmp = copy.join(format!(
        "{}.partial-{}",
        copied::FILE_NAME,
        std::process::id()
    ));
    std::fs::write(&tmp, record.to_toml()).map_err(|e| io(&tmp, e))?;
    std::fs::rename(&tmp, &at).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        io(&at, e)
    })
}
