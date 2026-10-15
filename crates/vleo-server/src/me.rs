//! Who is using the program: a key made, opened with its passphrase, and the
//! person the design's files register with it (docs/OPERATING_1_0.md, section
//! 14; docs/briefs/F1.md).
//!
//! The private half lives on the person's own computer, as `<person>.vkey` in
//! `VLEO_KEYS` (default `~/.vleo/keys`), never on the drive. An opened key is
//! held in this program's memory only, and dropped when it is closed or the
//! program ends. Who the key belongs to is read from the files, through the
//! chain: the programme's file, checked against the anchor START HERE names,
//! then the systems' file, then each subsystem's group file through the
//! systems' file that mounts it. Only those two files mount: a group file
//! that mounts another is not read as its parent. A key nobody registers
//! opens nothing, and says why.
//!
//! The passphrase arrives in a POST body on the loopback connection, is used
//! once to unlock the key, and is never stored, logged or echoed.

use super::*;
use std::sync::Mutex;
use vleo_files::chain::{self, Me, Registry};
use vleo_files::keys::{LockedKey, SigningKey};
use vleo_files::meta::Kind as FileKind;
use vleo_files::model::File;
use vleo_files::sqlite;
use vleo_files::{copied, people};

/// The key opened in this program, and who it says they are.
struct Opened {
    key: SigningKey,
    me: Me,
    fingerprint: String,
}

static OPEN: Mutex<Option<Opened>> = Mutex::new(None);

/// The shortest passphrase a key is locked with.
const SHORTEST: usize = 10;

/// Wrong passphrases for one key file before `open` makes it wait, and how
/// long: a person mistypes a few times; a program guessing does not stop.
const TRIES: u32 = 5;
const PAUSE: std::time::Duration = std::time::Duration::from_secs(30);

/// Each key file's wrong passphrases in a row, and when the last was.
static WRONG: Mutex<std::collections::BTreeMap<String, (u32, std::time::Instant)>> =
    Mutex::new(std::collections::BTreeMap::new());

/// The word this launch gives the page it opens, in the address's fragment,
/// which a browser never sends. A request that opens or uses a key, or writes
/// the person's copy, carries it back as `session`: the page the program
/// opened can, and another person's program on the same computer, which can
/// reach the loopback port too, cannot.
static SESSION: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// This launch's word (`SESSION`), made the first time it is asked for.
pub fn session() -> &'static str {
    SESSION.get_or_init(|| {
        vleo_files::keys::random_bytes::<16>()
            .map(|b| vleo_files::keys::hex(&b))
            .unwrap_or_else(|_| {
                // No randomness from the system: no word, so no request can
                // carry it and nothing that needs it is done.
                String::new()
            })
    })
}

/// Whether the request carries this launch's word.
pub(super) fn carries_session(params: &str) -> Result<(), String> {
    let given = param(params, "session").map(decode).unwrap_or_default();
    let word = session();
    // Compared in time that does not depend on where the first difference is.
    let differs = given
        .bytes()
        .zip(word.bytes())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b));
    if word.is_empty() || given.len() != word.len() || differs != 0 {
        return Err("this request does not carry the word this launch gave its page: a key is opened or used, and the copy written, only from the page the program opened".into());
    }
    Ok(())
}

/// Where this computer keeps its people's keys, and the anchors it accepted,
/// one for each drive.
pub(super) fn keys_dir() -> PathBuf {
    match std::env::var_os("VLEO_KEYS").filter(|v| !v.is_empty()) {
        Some(d) => PathBuf::from(d),
        None => vleo_data::home()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".vleo/keys"),
    }
}

/// The folder the people and groups are read from: the person's working copy
/// when there is one, else the drive the program was started on, else the
/// design beside it.
pub(super) fn database(ctx: &Ctx) -> PathBuf {
    let copy = copy::dir(ctx);
    if copy.join(copied::FILE_NAME).is_file() {
        return copy;
    }
    match std::env::var_os("VLEO_DRIVE").filter(|v| !v.is_empty()) {
        Some(d) => PathBuf::from(d),
        None => ctx.root.join("design"),
    }
}

/// A file name for a person's key: their name in lower case, its letters and
/// digits kept and every run of anything else made one hyphen, with a short
/// digest of the name when it has letters outside ASCII.
fn file_name(person: &str) -> String {
    let mut out = String::new();
    for c in person.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let mut out = out.trim_end_matches('-').to_string();
    // A name with letters outside ASCII keeps what it can, and a short digest
    // of the whole name, so two such names never share a file.
    if out.is_empty() || !person.is_ascii() {
        let d = vleo_files::keys::hex(&vleo_files::keys::sha256(person.as_bytes()));
        if !out.is_empty() {
            out.push('-');
        }
        out.push_str(&d[..12]);
    }
    format!("{out}.vkey")
}

/// A key file, or the keys folder (`mode` 0o700), readable by its person
/// alone, where the system says who that is. The folder is closed before a
/// key is written into it, so the key is never readable by another while it
/// is being written either.
fn private(path: &Path, mode: u32) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode));
    }
    #[cfg(not(unix))]
    let _ = (path, mode);
}

pub(super) fn refusal(message: &str) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", false);
    j.str_field("message", message);
    j.raw("}");
    j.0
}

/// `POST /v1/key/make`: a new key for `name`, locked by `passphrase`, written
/// on this computer. Making a key registers nobody: its public half goes to
/// whoever registers the person, who adds it to their file.
pub(super) fn make(params: &str) -> String {
    let name = param(params, "name").map(decode).unwrap_or_default();
    let name = name.trim();
    let passphrase =
        zeroize::Zeroizing::new(param(params, "passphrase").map(decode).unwrap_or_default());
    if name.is_empty() {
        return refusal("a key is made for a person: give your name as the files will register it");
    }
    if passphrase.chars().count() < SHORTEST {
        return refusal(&format!(
            "a passphrase of at least {SHORTEST} characters locks the key: it is the only thing between someone who copies the file and your signature"
        ));
    }
    let dir = keys_dir();
    let path = dir.join(file_name(name));
    if path.exists() {
        return refusal(&format!(
            "{} already holds a key for {name}: a key is never written over. Open that one, or move it aside first",
            path.display()
        ));
    }
    let made = (|| -> Result<(String, String, String), vleo_files::Error> {
        let key = SigningKey::generate()?;
        let salt = vleo_files::keys::random_bytes::<16>()?;
        let nonce = vleo_files::keys::random_bytes::<12>()?;
        let locked = key.lock(name, &passphrase, salt, nonce, vleo_files::keys::ITERATIONS)?;
        let mut f = File::new(FileKind::Key, concat!("vleo ", env!("CARGO_PKG_VERSION")));
        f.meta.insert("person".into(), name.into());
        f.locked_keys.push(locked.to_row());
        std::fs::create_dir_all(&dir).map_err(|e| {
            vleo_files::Error::new(vleo_files::ErrorKind::Io, format!("{}: {e}", dir.display()))
        })?;
        private(&dir, 0o700);
        sqlite::write(&f, &path)?;
        private(&path, 0o600);
        Ok((
            key.public().to_hex(),
            key.public().fingerprint(),
            people::proof(&key, name),
        ))
    })();
    match made {
        Ok((public, fingerprint, proof)) => {
            let mut j = Json::new();
            j.raw("{");
            j.bool_field("ok", true);
            j.str_field("person", name);
            j.str_field("file", &path.display().to_string());
            j.str_field("public_key", &public);
            j.str_field("fingerprint", &fingerprint);
            j.str_field("proof", &proof);
            j.str_field(
                "message",
                "Your key is made and locked. Give the public key and its proof to whoever registers you: the programme manager, or your subsystem engineer.",
            );
            j.raw("}");
            j.0
        }
        Err(e) => refusal(e.message()),
    }
}

/// `GET /v1/keys`: the keys this computer holds, by file and person.
/// The key files in the keys folder, and whose each is: the only files
/// `open` reaches for.
fn keys_here() -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = std::fs::read_dir(keys_dir())
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "vkey"))
        .filter_map(|p| {
            let person = sqlite::read(&p).ok()?.meta.get("person").cloned()?;
            Some((p.file_name()?.to_string_lossy().into_owned(), person))
        })
        .collect();
    found.sort();
    found
}

/// `GET /v1/keys`: the keys on this computer.
pub(super) fn list() -> String {
    let found = keys_here();
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("folder", &keys_dir().display().to_string());
    j.key("keys").raw("[");
    for (i, (file, person)) in found.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("file", file);
        j.str_field("person", person);
        j.raw("}");
    }
    j.raw("]}");
    j.0
}

/// What START HERE names, and whether this computer accepted it for this
/// drive before.
enum Anchor {
    Accepted(String),
    FirstUse(String),
}

/// Why the files could not be read: this computer has not accepted the
/// drive's anchor yet, or anything else.
pub(super) enum NotRead {
    /// The fingerprint START HERE names, and the drive it would be kept for.
    FirstUse(String, String),
    Why(String),
}

/// The anchors this computer accepted, one line each: the fingerprint, then
/// the drive it was accepted for.
fn anchors_file() -> PathBuf {
    keys_dir().join("anchors")
}

/// The drive as this computer names it, so each drive keeps its own anchor.
/// A working copy is named by the drive it was copied from, so copying again
/// asks nobody to accept the same anchor twice.
/// A record beside the folder that does not read is refused, never taken for
/// a folder of its own. The record is read only beside `copy`, the working
/// copy the program took: one found on a drive opened as it is says nothing,
/// for anyone who can write to that drive could have put it there.
fn drive_name(database: &Path, copy: &Path) -> Result<String, String> {
    let record = if database == copy {
        copy::record(database)?
    } else {
        None
    };
    let at = match record {
        Some(c) => PathBuf::from(c.drive),
        None => database.to_path_buf(),
    };
    Ok(std::fs::canonicalize(&at)
        .unwrap_or(at)
        .display()
        .to_string())
}

/// The anchor: the programme manager's key fingerprint START HERE names, the
/// first word of 64 hexadecimal digits in it. This computer remembers the one
/// it accepted for each drive, and refuses a different one: a changed anchor
/// comes with a note signed by the old key, which is not read here yet
/// (section 14).
fn anchor(database: &Path, copy: &Path) -> Result<Anchor, String> {
    let start = ["START HERE.md", "START HERE", "START_HERE.md"]
        .iter()
        .map(|n| database.join(n))
        .find(|p| p.is_file())
        .ok_or_else(|| {
            format!(
                "{} holds no START HERE naming the programme manager's key, so no signature can be checked",
                database.display()
            )
        })?;
    let text = std::fs::read_to_string(&start).map_err(|e| format!("{}: {e}", start.display()))?;
    let named = text
        .split(|c: char| !c.is_ascii_hexdigit())
        .find(|w| w.len() == 64)
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| format!("{} names no key fingerprint", start.display()))?;
    let kept = match std::fs::read_to_string(anchors_file()) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => {
            return Err(format!(
            "{}: {e}: the anchors this computer accepted cannot be read, so none is taken afresh",
            anchors_file().display()
        ))
        }
    };
    let drive = drive_name(database, copy)?;
    let before = kept.lines().find_map(|l| {
        let (fingerprint, d) = l.split_once(' ')?;
        (d == drive).then(|| fingerprint.to_string())
    });
    match before {
        Some(b) if b == named => Ok(Anchor::Accepted(named)),
        Some(b) => Err(format!(
            "START HERE names the programme manager's key {}…, but this computer accepted {}… for this drive before: a changed anchor comes with a note signed by the old key. Ask the programme manager",
            &named[..12],
            b.get(..12).unwrap_or("")
        )),
        None => Ok(Anchor::FirstUse(named)),
    }
}

/// `fingerprint` remembered as this drive's anchor.
fn accept(database: &Path, copy: &Path, fingerprint: &str) -> Result<(), String> {
    let drive = drive_name(database, copy)?;
    use std::io::Write as _;
    let at = anchors_file();
    std::fs::create_dir_all(keys_dir())
        .and_then(|_| {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&at)
        })
        .and_then(|mut f| writeln!(f, "{fingerprint} {drive}"))
        .map_err(|e| format!("{}: {e}", at.display()))
}

/// The files of a drive, opened through the chain.
pub(super) struct Chain {
    /// The fingerprint the programme's file opened under.
    pub(super) anchor: String,
    pub(super) programme: Registry,
    pub(super) systems: Option<Registry>,
    pub(super) groups: Vec<Registry>,
    pub(super) left_out: Vec<String>,
}

/// The programme's, the systems' and every group's files in `database`, opened
/// through the chain. A group file that does not open is left out, with why.
/// The first time this computer opens the drive, the anchor is remembered only
/// once the programme's file opens under it and the person has accepted it
/// (`accept_anchor`, the fingerprint they compared).
pub(super) fn registries(
    database: &Path,
    copy: &Path,
    accept_anchor: &str,
) -> Result<Chain, NotRead> {
    let (named, first) = match anchor(database, copy).map_err(NotRead::Why)? {
        Anchor::Accepted(n) => (n, false),
        Anchor::FirstUse(n) => (n, true),
    };
    let groups = database.join("groups");
    let read = |id: &str| -> Result<File, String> {
        let p = groups.join(id).join(format!("{id}.vgroup"));
        if !p.is_file() {
            return Err(format!(
                "{id} is mounted, but {} holds no {id}.vgroup",
                groups.join(id).display()
            ));
        }
        let f = sqlite::read(&p).map_err(|e| format!("{}: {}", p.display(), e.message()))?;
        match f.meta.get("group_id") {
            Some(g) if g == id => Ok(f),
            g => Err(format!(
                "{} says it is the group {:?}: a group's file lives in its own group's folder",
                p.display(),
                g.map_or("", String::as_str)
            )),
        }
    };
    let programme = read("programme")
        .and_then(|f| chain::open_programme(f, &named).map_err(|e| e.message().to_string()))
        .map_err(NotRead::Why)?;
    if first {
        if accept_anchor.trim().to_ascii_lowercase() != named {
            return Err(NotRead::FirstUse(
                named,
                drive_name(database, copy).map_err(NotRead::Why)?,
            ));
        }
        accept(database, copy, &named).map_err(NotRead::Why)?;
    }
    let mut left_out = Vec::new();
    let systems = match read("systems")
        .and_then(|f| chain::open_systems(&f, &programme).map_err(|e| e.message().to_string()))
    {
        Ok(s) => Some(s),
        Err(why) => {
            left_out.push(format!("systems: {why}"));
            None
        }
    };
    let mut ids: Vec<String> = std::fs::read_dir(&groups)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|id| id != "programme" && id != "systems")
        // A folder that holds no group file of its own name is not a group.
        .filter(|id| groups.join(id).join(format!("{id}.vgroup")).is_file())
        .collect();
    // A group a file mounts is read whether or not its folder holds its file:
    // one gone missing is refused and said, never passed over.
    for m in programme
        .file
        .mounts
        .iter()
        .chain(systems.iter().flat_map(|s| s.file.mounts.iter()))
    {
        if m.group_id != "systems" && !ids.contains(&m.group_id) {
            ids.push(m.group_id.clone());
        }
    }
    ids.sort();
    // Only the programme's and the systems' files mount.
    let parents: Vec<&Registry> = std::iter::once(&programme).chain(systems.iter()).collect();
    let mut opened = Vec::new();
    for id in ids {
        let opened_one = read(&id).and_then(|f| {
            let (parent, _) =
                chain::mounted_by(&id, &parents).map_err(|e| e.message().to_string())?;
            chain::open_group_in(&f, parent, &programme).map_err(|e| e.message().to_string())
        });
        match opened_one {
            Ok(g) => opened.push(g),
            Err(why) => left_out.push(format!("{id}: {why}")),
        }
    }
    Ok(Chain {
        anchor: named,
        programme,
        systems,
        groups: opened,
        left_out,
    })
}

/// The first time this computer opens a drive: the anchor shown, to be
/// compared and accepted.
fn first_use(fingerprint: &str, drive: &str) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", false);
    j.str_field("anchor", fingerprint);
    j.str_field("drive", drive);
    j.str_field(
        "message",
        &format!(
            "This computer has not opened the drive {drive} before. Its START HERE names the programme manager's key {fingerprint}. Compare it, every digit, with the one the programme manager gave you; if they agree, accept it, and this computer remembers it for that drive."
        ),
    );
    j.raw("}");
    j.0
}

fn me_json(
    open: Option<&Opened>,
    left_out: &[String],
    newer: &Result<Vec<String>, String>,
) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.bool_field("open", open.is_some());
    if let Err(why) = newer {
        j.str_field(
            "warning",
            &format!("The drive could not be checked for group files newer than your copy: {why}"),
        );
    }
    let newer: &[String] = newer.as_deref().unwrap_or(&[]);
    if !newer.is_empty() {
        j.key("drive_newer").raw("[");
        for (i, x) in newer.iter().enumerate() {
            if i > 0 {
                j.raw(",");
            }
            j.raw(&json::string(x));
        }
        j.raw("]");
        j.str_field(
            "warning",
            "The drive holds newer group files than your copy, a revocation among them perhaps: who your key belongs to is read from your copy. Copy again once your work is back.",
        );
    }
    if let Some(o) = open {
        j.str_field("name", &o.me.name);
        j.str_field("fingerprint", &o.fingerprint);
        j.key("roles").raw("[");
        for (i, (role, from)) in o.me.roles.iter().enumerate() {
            if i > 0 {
                j.raw(",");
            }
            j.raw("{");
            j.str_field("role", role);
            j.str_field("in", from);
            j.raw("}");
        }
        j.raw("]");
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
        list(&mut j, "owns", &o.me.owns);
        list(&mut j, "deputy_for", &o.me.deputy_for);
        j.key("nodes").raw("[");
        for (i, (group, block)) in o.me.nodes.iter().enumerate() {
            if i > 0 {
                j.raw(",");
            }
            j.raw("{");
            j.str_field("group", group);
            j.str_field("block", block);
            j.raw("}");
        }
        j.raw("]");
    } else {
        j.str_field(
            "message",
            "No key is open. Open yours with its passphrase, or make one.",
        );
    }
    j.key("left_out").raw("[");
    for (i, x) in left_out.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw(&json::string(x));
    }
    j.raw("]}");
    j.0
}

/// The group files the drive holds newer than the working copy, when
/// `database` is that copy; or why the drive could not be checked. Only the
/// group files are read, not the whole drive.
fn newer_on_drive(database: &Path, copy: &Path) -> Result<Vec<String>, String> {
    if database != copy {
        return Ok(Vec::new());
    }
    let Some(r) = copy::record(database)? else {
        return Ok(Vec::new());
    };
    let mut was = r.clone();
    was.files.retain(|p, _| p.ends_with(".vgroup"));
    Ok(was.changed(&copy::group_files(Path::new(&r.drive))?))
}

/// The page the program opens: a file only this person can read, in their
/// keys folder, that takes the browser to the address with this launch's word.
/// The word never goes in a program's arguments, which anyone on the computer
/// can read.
pub(super) fn launch_page(port: u16) -> Result<PathBuf, String> {
    let dir = keys_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    private(&dir, 0o700);
    let at = dir.join("open.html");
    let address = format!("http://127.0.0.1:{port}/app#session={}", session());
    let page = format!(
        "<!doctype html>\n<meta charset=\"utf-8\">\n<title>VLEO</title>\n<script>location.replace({});</script>\n<p>Opening <a href=\"http://127.0.0.1:{port}/app\">the program</a>.</p>\n",
        json::string(&address)
    );
    let _ = std::fs::remove_file(&at);
    std::fs::write(&at, page).map_err(|e| format!("{}: {e}", at.display()))?;
    private(&at, 0o600);
    Ok(at)
}

/// `POST /v1/key/open`: the key in `file` (a name in the keys folder),
/// unlocked by `passphrase`, and the person the files register with it. It is
/// held only if both hold.
pub(super) fn open(params: &str, ctx: &Ctx) -> String {
    if let Err(why) = carries_session(params) {
        return refusal(&why);
    }
    // Whatever happens next, nobody's key stays open from before.
    *OPEN.lock().unwrap_or_else(|p| p.into_inner()) = None;
    let file = param(params, "file").map(decode).unwrap_or_default();
    let passphrase =
        zeroize::Zeroizing::new(param(params, "passphrase").map(decode).unwrap_or_default());
    if !keys_here().iter().any(|(f, _)| *f == file) {
        return refusal("name the key by its file in the keys folder, as /v1/keys lists it");
    }
    let path = keys_dir().join(&file);
    let f = match sqlite::read(&path) {
        Ok(f) if f.kind().ok() == Some(FileKind::Key) => f,
        Ok(_) => return refusal(&format!("{file} is not a key file")),
        Err(e) => return refusal(&format!("{file}: {}", e.message())),
    };
    {
        let wrong = WRONG.lock().unwrap_or_else(|p| p.into_inner());
        if let Some((n, last)) = wrong.get(&file) {
            let waited = last.elapsed();
            if *n >= TRIES && waited < PAUSE {
                return refusal(&format!(
                    "{n} wrong passphrases in a row for {file}: try again in {} seconds",
                    (PAUSE - waited).as_secs() + 1
                ));
            }
        }
    }
    let unlocked = f
        .locked_keys
        .first()
        .ok_or_else(|| {
            vleo_files::Error::new(
                vleo_files::ErrorKind::Malformed,
                "the key file holds no key",
            )
        })
        .and_then(LockedKey::from_row)
        .and_then(|k| k.unlock(&passphrase));
    let key = {
        let mut wrong = WRONG.lock().unwrap_or_else(|p| p.into_inner());
        match unlocked {
            Ok(k) => {
                wrong.remove(&file);
                k
            }
            Err(e) => {
                let n = wrong.get(&file).map_or(0, |(n, _)| *n) + 1;
                wrong.insert(file.clone(), (n, std::time::Instant::now()));
                return refusal(e.message());
            }
        }
    };
    let accept_anchor = param(params, "accept_anchor")
        .map(decode)
        .unwrap_or_default();
    let c = match registries(&database(ctx), &copy::dir(ctx), &accept_anchor) {
        Ok(c) => c,
        Err(NotRead::FirstUse(fingerprint, drive)) => return first_use(&fingerprint, &drive),
        Err(NotRead::Why(why)) => {
            return refusal(&format!(
                "Your key opened, but who it belongs to cannot be read: {why}"
            ))
        }
    };
    let refs: Vec<&Registry> = c.groups.iter().collect();
    let left_out = c.left_out;
    match chain::whoami(
        &key.public().to_hex(),
        &results::now_utc(),
        &c.programme,
        c.systems.as_ref(),
        &refs,
    ) {
        Ok(me) => {
            let opened = Opened {
                fingerprint: key.public().fingerprint(),
                key,
                me,
            };
            let newer = newer_on_drive(&database(ctx), &copy::dir(ctx));
            let out = me_json(Some(&opened), &left_out, &newer);
            *OPEN.lock().unwrap_or_else(|p| p.into_inner()) = Some(opened);
            out
        }
        Err(e) => {
            let public = key.public().to_hex();
            let known = std::iter::once(&c.programme)
                .chain(c.systems.iter())
                .chain(c.groups.iter())
                .any(|r| r.file.keys.iter().any(|k| k.public_key == public));
            // The refused files that list this key: read without checking,
            // only to say which, never to count it. A file planted there can at
            // worst withhold the handover, never give one.
            let db = database(ctx);
            let in_refused: Vec<&String> = left_out
                .iter()
                .filter(|why| {
                    let id = why.split(':').next().unwrap_or("");
                    sqlite::read(&db.join("groups").join(id).join(format!("{id}.vgroup")))
                        .is_ok_and(|f| f.keys.iter().any(|k| k.public_key == public))
                })
                .collect();
            let refused = if in_refused.is_empty() {
                String::new()
            } else {
                format!(
                    " Your key is in a file that is refused, so who it registers is not read: {}.",
                    in_refused
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join("; ")
                )
            };
            let mut j = Json::new();
            j.raw("{");
            j.bool_field("ok", false);
            // Only a key no file lists, refused or not, is one to hand over: a
            // revoked key, one registered twice, or one whose group's file is
            // refused is not fixed by registering it again.
            if !known && in_refused.is_empty() {
                let person = f.meta.get("person").cloned().unwrap_or_default();
                j.str_field("person", &person);
                j.str_field("public_key", &public);
                j.str_field("proof", &people::proof(&key, &person));
                j.str_field(
                    "message",
                    &format!(
                        "Your key opened, but it opens nothing here: {}. Give whoever registers you its public key and proof, below, for the name {person}.",
                        e.message()
                    ),
                );
            } else {
                j.str_field(
                    "message",
                    &format!(
                        "Your key opened, but it opens nothing here: {}.{refused}",
                        e.message()
                    ),
                );
            }
            j.raw("}");
            j.0
        }
    }
}

/// `POST /v1/key/close`: the open key dropped from memory.
pub(super) fn close() -> String {
    *OPEN.lock().unwrap_or_else(|p| p.into_inner()) = None;
    "{\"ok\":true}".into()
}

/// `GET /v1/me`: who is using the program, or that no key is open.
pub(super) fn me() -> String {
    let open = OPEN.lock().unwrap_or_else(|p| p.into_inner());
    me_json(open.as_ref(), &[], &Ok(Vec::new()))
}

/// The open key and who it is, for whatever signs with it: a change to whom a
/// file registers (`people.rs`), and the signing F3 and F5 add.
pub(super) fn with_key<T>(f: impl FnOnce(&SigningKey, &Me) -> T) -> Option<T> {
    let open = OPEN.lock().unwrap_or_else(|p| p.into_inner());
    open.as_ref().map(|o| f(&o.key, &o.me))
}
