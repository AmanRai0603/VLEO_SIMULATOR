//! The person's working copy of the database (docs/PLAN_F.md, decision 3).
//!
//! The database lives on the shared drive. The program copies the latest of
//! it into the folder beside itself, `database/` (or `VLEO_COPY`), and works
//! on that copy; it writes `copied.toml` beside it, naming the drive and every
//! file's SHA-256 as copied (`vleo_files::copied`). A copy is never taken over
//! work in the old copy that has not gone back: the files whose bytes are not
//! what was copied are named, and nothing is written. Putting files back on
//! the drive is F3's (docs/briefs/F3.md, F3-c), and reads the same record.
//!
//! The copy is taken whole into a folder beside the old one, then swapped in,
//! so a copy cut short leaves the old copy as it was. The old copy is checked
//! again once it is moved aside, and put back if anything wrote into it
//! meanwhile; it is kept beside the new one as `.<copy>.before`, never
//! deleted. One program at a time copies or writes the copy: a lock file
//! beside it says which is.
//!
//! Only folders and ordinary files are copied. A link, a device or a pipe on
//! the drive is refused by name, and so is a key: a key never lives on the
//! drive. Each file is opened and checked to be the one that was listed, and
//! inside the drive, before its bytes are taken.

use super::*;
use std::collections::BTreeMap;
use vleo_files::copied::{self, Copied};

/// Where the working copy lives: `VLEO_COPY`, else `database/` beside the
/// program.
pub(super) fn dir(ctx: &Ctx) -> PathBuf {
    match std::env::var_os("VLEO_COPY").filter(|v| !v.is_empty()) {
        Some(d) => PathBuf::from(d),
        None => ctx.root.join("database"),
    }
}

/// The record beside the copy, if the program took one.
pub(super) fn record(copy: &Path) -> Result<Option<Copied>, String> {
    let at = copy.join(copied::FILE_NAME);
    match std::fs::read_to_string(&at) {
        Ok(t) => Copied::from_toml(&t)
            .map(Some)
            .map_err(|e| format!("{}: {}", at.display(), e.message())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: {e}", at.display())),
    }
}

/// Every file under `folder`, by its path inside it, with `/` between parts,
/// and the SHA-256 of its bytes. The record itself is not among them. A link,
/// or a name the record could not hold, is refused by name rather than
/// followed or skipped.
pub(super) fn digests(folder: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    let mut todo = vec![PathBuf::new()];
    while let Some(rel) = todo.pop() {
        let here = folder.join(&rel);
        let entries = std::fs::read_dir(&here).map_err(|e| format!("{}: {e}", here.display()))?;
        for e in entries {
            let e = e.map_err(|e| format!("{}: {e}", here.display()))?;
            let name = e.file_name().to_string_lossy().into_owned();
            let path = rel.join(&name);
            let shown = path
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            if shown == copied::FILE_NAME {
                continue;
            }
            let kind = e
                .file_type()
                .map_err(|e| format!("{}: {e}", folder.join(&path).display()))?;
            if kind.is_symlink() {
                return Err(format!(
                    "{shown} is a link: a database holds files, and a link is never followed"
                ));
            }
            if !kind.is_dir() && !kind.is_file() {
                return Err(format!(
                    "{shown} is neither a folder nor a file: a database holds only those"
                ));
            }
            if name.to_ascii_lowercase().ends_with(".vkey") {
                return Err(format!(
                    "{shown} is a key: a key lives on its person's own computer, never in a database"
                ));
            }
            if !copied::path_ok(&shown) {
                return Err(format!("{shown:?} is not a name the database can hold"));
            }
            if kind.is_dir() {
                todo.push(path);
            } else {
                let bytes = std::fs::read(folder.join(&path))
                    .map_err(|e| format!("{}: {e}", folder.join(&path).display()))?;
                out.insert(shown, copied::digest(&bytes));
            }
        }
    }
    Ok(out)
}

/// What START HERE is called, as `me::anchor` looks for it.
const START_HERE: &[&str] = &["START HERE.md", "START HERE", "START_HERE.md"];

/// Whether `folder` is a database: START HERE, and a `groups` folder.
fn is_database(folder: &Path) -> bool {
    START_HERE.iter().any(|n| folder.join(n).is_file()) && folder.join("groups").is_dir()
}

/// `POST /v1/copy`: the database on the drive (`drive`, else `VLEO_DRIVE`)
/// copied into the working copy, and the record written beside it.
pub(super) fn take(params: &str, ctx: &Ctx) -> String {
    if let Err(why) = me::carries_session(params) {
        return me::refusal(&why);
    }
    let drive = param(params, "drive")
        .map(decode)
        .filter(|d| !d.trim().is_empty())
        .or_else(|| {
            std::env::var("VLEO_DRIVE")
                .ok()
                .filter(|d| !d.trim().is_empty())
        });
    let Some(drive) = drive.map(PathBuf::from) else {
        return me::refusal("name the drive folder to copy the database from");
    };
    if !is_database(&drive) {
        return me::refusal(&format!(
            "{} is not a database: it holds no START HERE and no groups folder",
            drive.display()
        ));
    }
    let copy = dir(ctx);
    let _held = match lock(&copy) {
        Ok(l) => l,
        Err(why) => return me::refusal(&why),
    };
    match record(&copy) {
        Ok(Some(r)) => match digests(&copy) {
            Ok(now) => {
                let changed = r.changed(&now);
                if !changed.is_empty() {
                    return me::refusal(&format!(
                        "the copy holds work that has not gone back to the drive, so it is never copied over: {}. Put it back first, or move the copy aside",
                        shortlist(&changed)
                    ));
                }
            }
            Err(why) => return me::refusal(&why),
        },
        Ok(None) => {
            if std::fs::read_dir(&copy).is_ok_and(|mut d| d.next().is_some()) {
                return me::refusal(&format!(
                    "{} holds files the program did not copy, so it is never copied over: move them aside first",
                    copy.display()
                ));
            }
        }
        Err(why) => return me::refusal(&why),
    }
    match copy_whole(&drive, &copy) {
        Ok(r) => {
            let mut j = Json::new();
            j.raw("{");
            j.bool_field("ok", true);
            j.str_field("drive", &r.drive);
            j.str_field("copy", &copy.display().to_string());
            j.str_field("copied_at", &r.copied_at);
            j.key("files").raw(&r.files.len().to_string());
            j.str_field(
                "message",
                &format!(
                    "The database is copied from the drive: {} files. Your work is on this copy until it goes back.",
                    r.files.len()
                ),
            );
            j.raw("}");
            j.0
        }
        Err(why) => me::refusal(&format!("the database is not copied: {why}")),
    }
}

/// The copy's lock, taken, or why not: another program holds it. It is the
/// library's (`vleo_files::save::lock`), so a save through it and a copy here
/// never write the copy at once.
pub(super) fn lock(copy: &Path) -> Result<vleo_files::save::Lock, String> {
    vleo_files::save::lock(copy).map_err(|e| e.message().to_string())
}

/// The folder the copy lives in, and the copy's own name.
fn beside(copy: &Path) -> Result<(PathBuf, String), String> {
    let parent = copy
        .parent()
        .ok_or_else(|| format!("{} has no folder above it", copy.display()))?
        .to_path_buf();
    let name = copy
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "database".into());
    Ok((parent, name))
}

/// The bytes of the file `path` names inside `drive`, if it is the file that
/// was listed there: an ordinary file, opened without leaving the drive. A
/// folder swapped for a link after it was listed is caught here.
fn take_file(drive_root: &Path, drive: &Path, path: &str) -> Result<Vec<u8>, String> {
    use std::io::Read as _;
    let at = drive.join(path);
    let moved = || format!("{path} changed on the drive while it was copied: copy again");
    let listed = std::fs::symlink_metadata(&at).map_err(|e| format!("{}: {e}", at.display()))?;
    if !listed.file_type().is_file() {
        return Err(moved());
    }
    let real = std::fs::canonicalize(&at).map_err(|e| format!("{}: {e}", at.display()))?;
    if !real.starts_with(drive_root) {
        return Err(format!(
            "{path} leads outside the drive: a database holds files, and a link is never followed"
        ));
    }
    let mut f = std::fs::File::open(&real).map_err(|e| format!("{}: {e}", at.display()))?;
    let opened = f.metadata().map_err(|e| format!("{}: {e}", at.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if (opened.dev(), opened.ino()) != (listed.dev(), listed.ino()) {
            return Err(moved());
        }
    }
    if !opened.is_file() {
        return Err(moved());
    }
    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes)
        .map_err(|e| format!("{}: {e}", at.display()))?;
    Ok(bytes)
}

/// The drive's files into a fresh folder beside the copy, the record beside
/// them, then swapped in for the old copy, which is checked again and kept.
fn copy_whole(drive: &Path, copy: &Path) -> Result<Copied, String> {
    let (parent, name) = beside(copy)?;
    let incoming = parent.join(format!(".{name}.incoming-{}", std::process::id()));
    let previous = parent.join(format!(".{name}.previous-{}", std::process::id()));
    let before =
        |stamp: &str| parent.join(format!(".{name}.before-{}", stamp.replace([':', '-'], "")));
    let drive_root =
        std::fs::canonicalize(drive).map_err(|e| format!("{}: {e}", drive.display()))?;
    let _ = std::fs::remove_dir_all(&incoming);
    let result = (|| -> Result<Copied, String> {
        let files = digests(drive)?;
        let mut taken = BTreeMap::new();
        for path in files.keys() {
            let bytes = take_file(&drive_root, drive, path)?;
            let to = incoming.join(path);
            if let Some(p) = to.parent() {
                std::fs::create_dir_all(p).map_err(|e| format!("{}: {e}", p.display()))?;
            }
            std::fs::write(&to, &bytes).map_err(|e| format!("{}: {e}", to.display()))?;
            // The digest of what was written, not of what was listed: a file
            // that changed on the drive while it was copied is recorded as
            // the copy holds it.
            taken.insert(path.clone(), copied::digest(&bytes));
        }
        let r = Copied {
            drive: drive_root.display().to_string(),
            copied_at: results::now_utc(),
            files: taken,
        };
        std::fs::write(incoming.join(copied::FILE_NAME), r.to_toml())
            .map_err(|e| format!("{}: {e}", incoming.display()))?;
        Ok(r)
    })();
    let r = match result {
        Ok(r) => r,
        Err(why) => {
            let _ = std::fs::remove_dir_all(&incoming);
            return Err(why);
        }
    };
    if copy.exists() {
        std::fs::rename(copy, &previous).map_err(|e| {
            let _ = std::fs::remove_dir_all(&incoming);
            format!("{}: {e}", copy.display())
        })?;
        // Checked again where nothing reaches it: work written into the old
        // copy since it was first checked puts it back, and nothing is taken.
        let still = match record(&previous) {
            Ok(Some(old)) => digests(&previous).map(|now| old.changed(&now)),
            Ok(None) => Ok(Vec::new()),
            Err(why) => Err(why),
        };
        let refused = match still {
            Ok(changed) if changed.is_empty() => None,
            Ok(changed) => Some(format!(
                "work was written into the copy while it was being copied, so the old copy is kept as it was: {}",
                shortlist(&changed)
            )),
            Err(why) => Some(why),
        };
        if let Some(why) = refused {
            let _ = std::fs::rename(&previous, copy);
            let _ = std::fs::remove_dir_all(&incoming);
            return Err(why);
        }
    }
    if let Err(e) = std::fs::rename(&incoming, copy) {
        // Put the old copy back where it was.
        let _ = std::fs::rename(&previous, copy);
        let _ = std::fs::remove_dir_all(&incoming);
        return Err(format!("{}: {e}", copy.display()));
    }
    if previous.exists() {
        // Kept, never deleted: each copy before this one, checked to hold
        // nothing that had not gone back, under the moment it was taken.
        let stamp = record(&previous)
            .ok()
            .flatten()
            .map_or_else(results::now_utc, |old| old.copied_at);
        // Two copies in the same second keep both: the second gets a number.
        let mut kept = before(&stamp);
        let mut n = 1;
        while kept.exists() {
            n += 1;
            kept = PathBuf::from(format!("{}-{n}", before(&stamp).display()));
        }
        let _ = std::fs::rename(&previous, kept);
    }
    Ok(r)
}
/// The first few of `paths`, and how many more.
fn shortlist(paths: &[String]) -> String {
    let shown: Vec<&str> = paths.iter().take(8).map(String::as_str).collect();
    if paths.len() > shown.len() {
        format!(
            "{} and {} more",
            shown.join(", "),
            paths.len() - shown.len()
        )
    } else {
        shown.join(", ")
    }
}

/// `GET /v1/copy`: where the working copy is, where it came from and when,
/// and what changed since, here (the person's own work) and on the drive
/// (everyone else's).
pub(super) fn state(ctx: &Ctx) -> String {
    let copy = dir(ctx);
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("copy", &copy.display().to_string());
    let r = match record(&copy) {
        Ok(Some(r)) => r,
        Ok(None) => {
            j.bool_field("copied", false);
            j.str_field(
                "message",
                "No copy of the database yet: copy it from the drive to work on it.",
            );
            j.raw("}");
            return j.0;
        }
        Err(why) => {
            j.bool_field("copied", false);
            j.bool_field("unreadable", true);
            j.str_field("message", &why);
            j.raw("}");
            return j.0;
        }
    };
    j.bool_field("copied", true);
    j.str_field("drive", &r.drive);
    let mut notes: Vec<String> = Vec::new();
    if let Some(started) = std::env::var_os("VLEO_DRIVE").filter(|v| !v.is_empty()) {
        let started = std::fs::canonicalize(&started).unwrap_or_else(|_| PathBuf::from(&started));
        if started != Path::new(&r.drive) {
            notes.push(format!(
                "This copy came from {}, but the program was started on the drive {}.",
                r.drive,
                started.display()
            ));
        }
    }
    j.str_field("copied_at", &r.copied_at);
    let list = |j: &mut Json, k: &str, v: &[String]| {
        j.key(k).raw("[");
        for (i, x) in v.iter().enumerate() {
            if i > 0 {
                j.raw(",");
            }
            j.raw(&json::string(x));
        }
        j.raw("]");
    };
    match digests(&copy) {
        Ok(now) => list(&mut j, "changed_here", &r.changed(&now)),
        Err(why) => {
            list(&mut j, "changed_here", &[]);
            notes.push(why);
        }
    }
    match digests(Path::new(&r.drive)) {
        Ok(there) => {
            j.bool_field("drive_reached", true);
            list(&mut j, "changed_on_drive", &r.changed(&there));
        }
        Err(_) => {
            j.bool_field("drive_reached", false);
            list(&mut j, "changed_on_drive", &[]);
        }
    }
    if !notes.is_empty() {
        j.str_field("message", &notes.join(" "));
    }
    j.raw("}");
    j.0
}

/// Each group's file in `folder`, `groups/<id>/<id>.vgroup` and any other
/// `.vgroup` beside it, by its path inside the database, with the SHA-256 of
/// its bytes: what is read to tell whether the drive holds newer group files
/// than a copy, without reading the whole drive.
pub(super) fn group_files(folder: &Path) -> Result<BTreeMap<String, String>, String> {
    let groups = folder.join("groups");
    let mut out = BTreeMap::new();
    for g in std::fs::read_dir(&groups).map_err(|e| format!("{}: {e}", groups.display()))? {
        let g = g.map_err(|e| format!("{}: {e}", groups.display()))?;
        if !g.file_type().is_ok_and(|t| t.is_dir()) {
            continue;
        }
        let id = g.file_name().to_string_lossy().into_owned();
        for f in std::fs::read_dir(g.path()).map_err(|e| format!("{}: {e}", g.path().display()))? {
            let f = f.map_err(|e| format!("{}: {e}", g.path().display()))?;
            let name = f.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".vgroup") || !f.file_type().is_ok_and(|t| t.is_file()) {
                continue;
            }
            let bytes =
                std::fs::read(f.path()).map_err(|e| format!("{}: {e}", f.path().display()))?;
            out.insert(format!("groups/{id}/{name}"), copied::digest(&bytes));
        }
    }
    Ok(out)
}
