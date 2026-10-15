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
//! so a copy cut short leaves the old copy as it was.

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

/// The drive's files into a fresh folder beside the copy, the record beside
/// them, then swapped in for the old copy.
fn copy_whole(drive: &Path, copy: &Path) -> Result<Copied, String> {
    let parent = copy
        .parent()
        .ok_or_else(|| format!("{} has no folder above it", copy.display()))?;
    std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    let name = copy
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "database".into());
    let incoming = parent.join(format!(".{name}.incoming-{}", std::process::id()));
    let previous = parent.join(format!(".{name}.previous-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&incoming);
    let result = (|| -> Result<Copied, String> {
        let files = digests(drive)?;
        let mut taken = BTreeMap::new();
        for path in files.keys() {
            let from = drive.join(path);
            let to = incoming.join(path);
            if let Some(p) = to.parent() {
                std::fs::create_dir_all(p).map_err(|e| format!("{}: {e}", p.display()))?;
            }
            let bytes = std::fs::read(&from).map_err(|e| format!("{}: {e}", from.display()))?;
            std::fs::write(&to, &bytes).map_err(|e| format!("{}: {e}", to.display()))?;
            // The digest of what was written, not of what was listed: a file
            // that changed on the drive while it was copied is recorded as
            // the copy holds it.
            taken.insert(path.clone(), copied::digest(&bytes));
        }
        let r = Copied {
            drive: std::fs::canonicalize(drive)
                .unwrap_or_else(|_| drive.to_path_buf())
                .display()
                .to_string(),
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
    }
    if let Err(e) = std::fs::rename(&incoming, copy) {
        // Put the old copy back where it was.
        let _ = std::fs::rename(&previous, copy);
        let _ = std::fs::remove_dir_all(&incoming);
        return Err(format!("{}: {e}", copy.display()));
    }
    let _ = std::fs::remove_dir_all(&previous);
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
            j.str_field("message", &why);
            j.raw("}");
            return j.0;
        }
    };
    j.bool_field("copied", true);
    j.str_field("drive", &r.drive);
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
            j.str_field("message", &why);
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
    j.raw("}");
    j.0
}
