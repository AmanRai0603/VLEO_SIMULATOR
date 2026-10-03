//! The tool's database files: `design.vleo`, the design as one file, here;
//! `results.vleor`, saved results many in one file, in [`results`].
//!
//! `design.vleo` — the design as one file.
//!
//! The tree the loader reads is some seven thousand files: a folder per node
//! under `crates/vleo-mod-*/nodes/`, the layers, the cases and the source
//! list. A developer edits them as folders, because that is what review and
//! the gate work on. The tool a team runs needs none of that: it needs the
//! design, whole, as one thing that cannot be half-copied. So a release carries
//! it as one SQLite database, written from the tree by `xtask design`, and the
//! daemon reads it through the same interface it reads the folders through
//! (`vleo_sheet::files::Files`). Every check the loader makes is made the same
//! way on either, and a page served from the file is the page served from the
//! folders, byte for byte — which `compare` proves and the tests hold.
//!
//! The file holds each file whole, under its path in the repository, with its
//! SHA-256; a short `row` table lists the rows for a reader who wants them
//! without reading sheets; `meta` says what the file is and what it was built
//! from. Its schema is `design.sql`, beside this crate.

pub mod results;
mod sha;
pub use sha::sha256_hex;

use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use vleo_sheet::files::{self, Files};

/// The file's name, beside the tool's other files.
pub const FILE: &str = "design.vleo";
/// The format this code writes and the newest it reads.
pub const FORMAT: u32 = 1;
/// 'VLEO', the same identity every database the tools write carries.
const APP_ID: i64 = 1447838031;
const SCHEMA: &str = include_str!("../design.sql");

/// What went wrong, as a caller decides by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// The file system refused.
    Io,
    /// The tree does not load, so there is no design to write.
    Tree,
    /// Not the file asked for: not SQLite, not one the tools wrote, or another
    /// kind (a group's release opened as a design file).
    WrongFile,
    /// The file is a newer format than this code reads.
    Newer,
    /// SQLite refused.
    Database,
}

/// An error from this crate: its kind, and the sentence that says it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Error {
        Error {
            kind,
            message: message.into(),
        }
    }
    fn db(path: &Path, e: rusqlite::Error) -> Error {
        Error::new(ErrorKind::Database, format!("{}: {e}", path.display()))
    }
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

impl From<Error> for String {
    fn from(e: Error) -> String {
        e.message
    }
}

/// What a design file says about where it came from.
#[derive(Clone, Debug, Default)]
pub struct Stamp {
    /// The tool's version.
    pub tool: String,
    /// The commit the tree was at, if known.
    pub commit: String,
    /// When it was written, as the caller states it.
    pub built: String,
}

/// What `write` wrote.
#[derive(Clone, Debug)]
pub struct Written {
    pub files: usize,
    pub bytes: u64,
    pub rows: usize,
    pub fingerprint: String,
}

/// The fingerprint of a set of files: the SHA-256 of one line per file,
/// `<sha256>  <path>`, in path order — the same lines `sha256sum` prints.
pub fn fingerprint<'a>(hashes: impl IntoIterator<Item = (&'a str, &'a str)>) -> String {
    let mut lines = String::new();
    for (path, sha) in hashes {
        lines.push_str(sha);
        lines.push_str("  ");
        lines.push_str(path);
        lines.push('\n');
    }
    sha256_hex(lines.as_bytes())
}

/// Write the design file for the tree at `root` to `out`.
///
/// The tree must load: a design file of a tree the loader refuses would be a
/// tool that starts and shows nothing. The file is written beside `out` and
/// renamed into place, so a reader never meets half of one.
pub fn write(root: &Path, out: &Path, stamp: &Stamp) -> Result<Written, Error> {
    let tree = vleo_sheet::load_all(root).map_err(|e| {
        Error::new(
            ErrorKind::Tree,
            format!("the tree does not load: {}", e.message()),
        )
    })?;
    let paths = files::design_files(root)
        .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", root.display())))?;
    let part = out.with_extension("vleo.part");
    let _ = std::fs::remove_file(&part);
    if let Some(dir) = out.parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir)
                .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", dir.display())))?;
        }
    }
    let mut db = Connection::open(&part).map_err(|e| Error::db(&part, e))?;
    db.execute_batch(SCHEMA).map_err(|e| Error::db(&part, e))?;
    let mut hashes: Vec<(String, String)> = Vec::with_capacity(paths.len());
    let mut bytes = 0u64;
    {
        let tx = db.transaction().map_err(|e| Error::db(&part, e))?;
        {
            let mut put = tx
                .prepare("INSERT INTO file (path, sha256, bytes) VALUES (?1, ?2, ?3)")
                .map_err(|e| Error::db(&part, e))?;
            for rel in &paths {
                let p = root.join(rel);
                let b = std::fs::read(&p)
                    .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", p.display())))?;
                let sha = sha256_hex(&b);
                bytes += b.len() as u64;
                put.execute(params![rel, sha, b])
                    .map_err(|e| Error::db(&part, e))?;
                hashes.push((rel.clone(), sha));
            }
            let mut row = tx
                .prepare(
                    "INSERT INTO row (id, folder, layer, ord, parent, subsystem, kind, state, \
                     owner, label, question) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                )
                .map_err(|e| Error::db(&part, e))?;
            for sh in tree.ordered() {
                row.execute(params![
                    sh.id,
                    sh.folder,
                    sh.layer,
                    sh.order,
                    sh.parent,
                    sh.subsystem,
                    sh.kind,
                    sh.state,
                    sh.owner,
                    sh.label,
                    sh.question
                ])
                .map_err(|e| Error::db(&part, e))?;
            }
        }
        let fp = fingerprint(hashes.iter().map(|(p, s)| (p.as_str(), s.as_str())));
        for (k, v) in [
            ("file_kind", "design".to_string()),
            ("format", FORMAT.to_string()),
            ("tool", stamp.tool.clone()),
            ("commit", stamp.commit.clone()),
            ("built", stamp.built.clone()),
            ("rows", tree.sheets.len().to_string()),
            ("files", paths.len().to_string()),
            ("fingerprint", fp),
        ] {
            tx.execute(
                "INSERT INTO meta (key, value) VALUES (?1, ?2)",
                params![k, v],
            )
            .map_err(|e| Error::db(&part, e))?;
        }
        tx.commit().map_err(|e| Error::db(&part, e))?;
    }
    db.execute_batch("VACUUM")
        .map_err(|e| Error::db(&part, e))?;
    db.close().map_err(|(_, e)| Error::db(&part, e))?;
    std::fs::rename(&part, out)
        .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", out.display())))?;
    Ok(Written {
        files: paths.len(),
        bytes,
        rows: tree.sheets.len(),
        fingerprint: fingerprint(hashes.iter().map(|(p, s)| (p.as_str(), s.as_str()))),
    })
}

/// Open one of the tools' databases read-only, as the kind `kind` and no newer
/// than `newest`. Refuses a file that is not SQLite, one the tools did not
/// write, one of another kind (a group's release opened as a design), and one
/// from a newer tool — each by name, before anything is read from it.
pub(crate) fn open_ours(path: &Path, kind: &str, newest: u32) -> Result<Connection, Error> {
    let mut head = [0u8; 16];
    {
        use std::io::Read;
        let mut f = std::fs::File::open(path)
            .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", path.display())))?;
        if f.read_exact(&mut head).is_err() || &head[..15] != b"SQLite format 3" {
            return Err(Error::new(
                ErrorKind::WrongFile,
                format!("{}: not a database file", path.display()),
            ));
        }
    }
    let db = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| Error::db(path, e))?;
    let app: i64 = db
        .query_row("PRAGMA application_id", [], |r| r.get(0))
        .map_err(|e| Error::db(path, e))?;
    if app != APP_ID {
        return Err(Error::new(
            ErrorKind::WrongFile,
            format!("{}: not a database the VLEO tools wrote", path.display()),
        ));
    }
    let format: u32 = db
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| Error::db(path, e))?;
    if format > newest {
        return Err(Error::new(
            ErrorKind::Newer,
            format!(
                "{}: format {format}, newer than this tool reads ({newest}): use the newer tool",
                path.display()
            ),
        ));
    }
    let found: Option<String> = db
        .query_row("SELECT value FROM meta WHERE key = 'file_kind'", [], |r| {
            r.get(0)
        })
        .optional()
        .map_err(|_| {
            Error::new(
                ErrorKind::WrongFile,
                format!("{}: not a {kind} file", path.display()),
            )
        })?;
    if found.as_deref() != Some(kind) {
        return Err(Error::new(
            ErrorKind::WrongFile,
            format!(
                "{}: a {} file, not a {kind} file",
                path.display(),
                found.as_deref().unwrap_or("VLEO")
            ),
        ));
    }
    Ok(db)
}

/// An open design file, read as the tree it was written from.
///
/// Paths are answered as if the tree sat at `root`: the daemon keeps reading
/// `root/crates/vleo-mod-solar/nodes/…`, and the file answers for it. Which
/// paths exist is read once, on open; each file's bytes are read when asked.
pub struct Design {
    path: PathBuf,
    root: PathBuf,
    db: Mutex<Connection>,
    files: BTreeSet<String>,
    dirs: BTreeSet<String>,
    meta: BTreeMap<String, String>,
}

impl Design {
    /// Open `path`, answering for the tree at `root`. Refuses a file that is
    /// not SQLite, not ours, not a design file, or newer than this code.
    pub fn open(path: &Path, root: &Path) -> Result<Design, Error> {
        let db = open_ours(path, "design", FORMAT)?;
        let mut meta = BTreeMap::new();
        let mut files = BTreeSet::new();
        let mut dirs = BTreeSet::new();
        {
            let mut q = db
                .prepare("SELECT key, value FROM meta")
                .map_err(|e| Error::db(path, e))?;
            let rows = q
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .map_err(|e| Error::db(path, e))?;
            for r in rows {
                let (k, v) = r.map_err(|e| Error::db(path, e))?;
                meta.insert(k, v);
            }
            let mut q = db
                .prepare("SELECT path FROM file")
                .map_err(|e| Error::db(path, e))?;
            let rows = q
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(|e| Error::db(path, e))?;
            for r in rows {
                let p = r.map_err(|e| Error::db(path, e))?;
                let mut at = p.as_str();
                while let Some(i) = at.rfind('/') {
                    at = &at[..i];
                    if !dirs.insert(at.to_string()) {
                        break;
                    }
                }
                files.insert(p);
            }
        }
        Ok(Design {
            path: path.to_path_buf(),
            root: root.to_path_buf(),
            db: Mutex::new(db),
            files,
            dirs,
            meta,
        })
    }

    /// Where the file is.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// One `meta` value, or "" when the file does not say.
    pub fn meta(&self, key: &str) -> &str {
        self.meta.get(key).map(String::as_str).unwrap_or("")
    }

    /// Every file it holds, by repository path.
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.files.iter().map(String::as_str)
    }

    /// Each file's bytes against the SHA-256 written beside them, and all of
    /// them against the fingerprint. The first difference, named; `Ok` when
    /// the file is what it says it is.
    pub fn verify(&self) -> Result<(), Error> {
        let db = self.db.lock().unwrap_or_else(|p| p.into_inner());
        let mut q = db
            .prepare("SELECT path, sha256, bytes FROM file ORDER BY path")
            .map_err(|e| Error::db(&self.path, e))?;
        let mut rows = q.query([]).map_err(|e| Error::db(&self.path, e))?;
        let mut hashes = Vec::new();
        while let Some(r) = rows.next().map_err(|e| Error::db(&self.path, e))? {
            let p: String = r.get(0).map_err(|e| Error::db(&self.path, e))?;
            let s: String = r.get(1).map_err(|e| Error::db(&self.path, e))?;
            let b: Vec<u8> = r.get(2).map_err(|e| Error::db(&self.path, e))?;
            if sha256_hex(&b) != s {
                return Err(Error::new(
                    ErrorKind::WrongFile,
                    format!(
                        "{}: {p} is not the file it was written as",
                        self.path.display()
                    ),
                ));
            }
            hashes.push((p, s));
        }
        let fp = fingerprint(hashes.iter().map(|(p, s)| (p.as_str(), s.as_str())));
        if fp != self.meta("fingerprint") {
            return Err(Error::new(
                ErrorKind::WrongFile,
                format!(
                    "{}: its files do not give the fingerprint it carries",
                    self.path.display()
                ),
            ));
        }
        Ok(())
    }

    /// Every way this file and the tree at `root` differ: a file only in one,
    /// or in both with different bytes. Empty when the file is the tree.
    pub fn compare(&self, root: &Path) -> Result<Vec<String>, Error> {
        let on_disk = files::design_files(root)
            .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", root.display())))?;
        let mut out = Vec::new();
        let disk: BTreeSet<&str> = on_disk.iter().map(String::as_str).collect();
        for p in &self.files {
            if !disk.contains(p.as_str()) {
                out.push(format!("{p}: in the design file, not in the tree"));
            }
        }
        for p in &on_disk {
            if !self.files.contains(p) {
                out.push(format!("{p}: in the tree, not in the design file"));
                continue;
            }
            let held = self.bytes(p)?.unwrap_or_default();
            let have = std::fs::read(root.join(p))
                .map_err(|e| Error::new(ErrorKind::Io, format!("{p}: {e}")))?;
            if held != have {
                out.push(format!("{p}: differs"));
            }
        }
        Ok(out)
    }

    fn bytes(&self, rel: &str) -> Result<Option<Vec<u8>>, Error> {
        let db = self.db.lock().unwrap_or_else(|p| p.into_inner());
        db.query_row(
            "SELECT bytes FROM file WHERE path = ?1",
            params![rel],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| Error::db(&self.path, e))
    }

    /// `p` as a path in the file, if it is under the root this file answers for.
    fn rel(&self, p: &Path) -> Option<String> {
        let r = p.strip_prefix(&self.root).ok()?;
        let parts: Vec<String> = r
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        Some(parts.join("/"))
    }
}

impl Files for Design {
    fn read(&self, p: &Path) -> io::Result<Vec<u8>> {
        let missing = || io::Error::new(io::ErrorKind::NotFound, "not in the design file");
        let rel = self.rel(p).ok_or_else(missing)?;
        if !self.files.contains(&rel) {
            return Err(missing());
        }
        self.bytes(&rel)
            .map_err(|e| io::Error::other(e.message))?
            .ok_or_else(missing)
    }
    fn entries(&self, p: &Path) -> io::Result<Vec<PathBuf>> {
        let rel = self
            .rel(p)
            .filter(|r| r.is_empty() || self.dirs.contains(r))
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "not in the design file"))?;
        let prefix = if rel.is_empty() {
            String::new()
        } else {
            format!("{rel}/")
        };
        // Every directory holds a file somewhere below it, so the names under
        // `rel` are the next part of each file path that starts with it.
        let mut names = BTreeSet::new();
        for q in self.files.range(prefix.clone()..) {
            let Some(rest) = q.strip_prefix(&prefix) else {
                break;
            };
            names.insert(rest.split('/').next().unwrap_or(rest).to_string());
        }
        Ok(names.into_iter().map(|n| p.join(n)).collect())
    }
    fn is_dir(&self, p: &Path) -> bool {
        self.rel(p)
            .is_some_and(|r| r.is_empty() || self.dirs.contains(&r))
    }
    fn is_file(&self, p: &Path) -> bool {
        self.rel(p).is_some_and(|r| self.files.contains(&r))
    }
}

/// The design file beside the tool's files at `root`, if there is one.
pub fn beside(root: &Path) -> Option<PathBuf> {
    let p = root.join(FILE);
    p.is_file().then_some(p)
}
