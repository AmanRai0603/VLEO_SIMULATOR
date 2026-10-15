//! Today's design: the design as every group's latest sealed release makes it.
//!
//! docs/OPERATING_1_0.md, section 5: "The application builds today's design on
//! opening, from the drive: the latest sealed release of every group that
//! passes its checks; for a group whose latest is refused, its last good
//! release, marked red with the reason." Everyone who opens it builds it from
//! the same files, so everyone sees the same design, and the application says
//! which releases it used.
//!
//! The drive holds each group's releases at `groups/<group>/releases/`. Each
//! release found there is taken only when it passes, in this order:
//!
//! 1. it opens, as a sealed group release (`vleo_files::intake::Release`);
//! 2. it says it is the group whose folder it is in;
//! 3. its files are the ones sealed — every file's SHA-256 gives the
//!    fingerprint it was sealed with (`vleo_files::seal`);
//! 4. its content holds: every check today's intake makes of a release
//!    (`vleo_files::checks::check_release`), each error named;
//! 5. each of its computed nodes, as the node form its node engineer would
//!    have filled, plans against the design without a conflict or a refusal
//!    (`vleo_sheet::template`) — a method or results an assistant supplied, a
//!    change with no de-risking record, a case that is not in SI.
//!
//! A release that fails any of them is refused, by its reason, and the group's
//! next older release is tried. A group with none that passes keeps the
//! design's own sheets for its nodes, and says so. Nothing on the drive is
//! written: the releases are applied to the design in memory, sheet by sheet,
//! and the tree is read again through every check the loader makes.
//!
//! And, when the drive has a programme's file, each release's seal is checked
//! through the chain from the anchor its START HERE names (`vleo_files::chain`,
//! docs/briefs/F2.md): the programme's file opened under that anchor, the
//! systems' file through it, and the release through whichever of the two
//! mounts its group. A release from before 1.0 checks by the programme
//! manager's anchor of its fingerprint. A seal that does not check is
//! refused like any other reason, and the group's last good release taken in
//! its place. Whether this computer has accepted that anchor yet is said
//! beside each release taken, read when it is said (`Taken::chain_said`). A drive with no programme's file says so for
//! every release: nothing is taken as if it had been checked.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use vleo_files::chain::{self, Registry};
use vleo_files::intake::{self, Release};
use vleo_files::meta::ReleaseVersion;
use vleo_sheet::files::Files;

/// What today's design was built from.
pub struct Today {
    /// The design, as the loader reads it: the base with every release taken.
    pub files: Arc<dyn Files>,
    /// Every group with a release on the drive, by its id.
    pub groups: Vec<Group>,
    /// Files on the drive that are not taken, and why: anything in a
    /// `releases/` folder that is not a release, two files where there should
    /// be one.
    pub ignored: Vec<String>,
    /// What the releases taken wrote into the design, as one digest: every
    /// sheet replaced, by its path under the design's root and the hash of
    /// its bytes. Empty when none was taken. See [`Today::fingerprint`].
    pub written: String,
    /// The released design today's design is based on, as the systems' file
    /// names it (`based_on`, `base_version`), or why none is named.
    pub base: Base,
}

/// The released design a design is based on: by name and by count, and the
/// sentence that says it.
#[derive(Clone, Debug, Default)]
pub struct Base {
    pub based_on: String,
    pub base_version: Option<u64>,
    pub said: String,
}

/// The released design the systems' file on `drive` names as the one it is
/// based on. The systems' file is the main valve: what it is based on is what
/// today's design is based on.
fn base_of(drive: &Path) -> Base {
    let path = drive.join("groups/systems/systems.vgroup");
    if !path.is_file() {
        return Base {
            said: "the drive has no systems' file, so no released design is named as its base"
                .into(),
            ..Base::default()
        };
    }
    match vleo_files::sqlite::read(&path) {
        Err(e) => Base {
            said: format!(
                "the systems' file does not read, so its base is not known: {}",
                e.message()
            ),
            ..Base::default()
        },
        Ok(f) => {
            let based_on = f.meta.get("based_on").cloned().unwrap_or_default();
            let base_version = f.base_version();
            let said = match (based_on.is_empty(), base_version) {
                (true, Some(0) | None) => "based on no released design: none has been released yet".to_string(),
                (_, Some(n)) => format!("based on the released design {based_on}, the {}", nth(n)),
                (false, None) => format!(
                    "based on the released design {based_on}; its count is not kept in the systems' file"
                ),
            };
            Base {
                based_on,
                base_version,
                said,
            }
        }
    }
}

/// 1st, 2nd, 3rd, 4th, … 11th, 12th, 13th, 21st.
fn nth(n: u64) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

/// What today's design was built from, kept for the screens to say it: the
/// drive, every group with a release on it, and what was not taken.
#[derive(Clone, Debug)]
pub struct Report {
    pub drive: String,
    /// The fingerprint of today's design (`Today::fingerprint`).
    pub fingerprint: String,
    /// The released design it is based on.
    pub base: Base,
    pub groups: Vec<Group>,
    pub ignored: Vec<String>,
}

/// One group, and the release its part of today's design is.
#[derive(Clone, Debug)]
pub struct Group {
    pub id: String,
    /// The release taken, when one passes.
    pub taken: Option<Taken>,
    /// Every newer release refused, newest first, with why.
    pub refused: Vec<(String, String)>,
}

/// The release a group's part of today's design is.
#[derive(Clone, Debug)]
pub struct Taken {
    pub version: String,
    pub sealed: String,
    pub sealed_by: String,
    pub fingerprint: String,
    /// The nodes it changes in the design.
    pub changes: Vec<String>,
    /// What a person should know about it: nodes not in the design, a
    /// transcription taken as an assistant's relation.
    pub notes: Vec<String>,
    /// How its seal was checked: through the chain, and from which anchor, or
    /// why it could not be. Said with whether this computer has accepted
    /// that anchor by [`Taken::chain_said`].
    pub chain: String,
    /// Whether it was checked from the anchor START HERE names.
    pub anchored: bool,
    /// The released design it was based on, by name and by count
    /// (`base_version`); empty and none for a release from before they were
    /// kept.
    pub based_on: String,
    pub base_version: Option<u64>,
}

/// The chain the drive's releases are checked through. One is made for a
/// build, so its largest variant's size is no cost worth a box.
#[allow(clippy::large_enum_variant)]
enum Chain {
    /// The drive has no programme's file: nothing is checked, and each
    /// release says why.
    Absent(String),
    /// The drive has one, and it does not open: no release is taken.
    Broken(String),
    Open {
        programme: Registry,
        systems: Option<Registry>,
    },
}

/// The chain `drive` holds, read once for a build.
fn chain_of(drive: &Path) -> Chain {
    let file = drive.join("groups/programme/programme.vgroup");
    if !file.is_file() {
        return Chain::Absent(
            "the drive has no programme's file, so there is no anchor to check them from".into(),
        );
    }
    let (anchor, _) = match crate::me::drive_anchor(drive) {
        Ok(a) => a,
        Err(why) => return Chain::Broken(why),
    };
    let programme = match vleo_files::sqlite::read(&file)
        .map_err(|e| e.message().to_string())
        .and_then(|f| chain::open_programme(f, &anchor).map_err(|e| e.message().to_string()))
    {
        Ok(p) => p,
        Err(why) => {
            return Chain::Broken(format!(
                "the programme's file does not open under the anchor START HERE names: {why}"
            ))
        }
    };
    let systems = vleo_files::sqlite::read(&drive.join("groups/systems/systems.vgroup"))
        .ok()
        .and_then(|f| chain::open_systems(&f, &programme).ok());
    Chain::Open { programme, systems }
}

impl Chain {
    /// How `release`, of group `id`, checks through the chain: what to say of
    /// it when it does, or why it is refused.
    fn check(
        &self,
        id: &str,
        release: &vleo_files::model::File,
        sealed_by: &str,
    ) -> Result<String, String> {
        match self {
            Chain::Absent(why) => Ok(format!("its seal is not checked through the chain: {why}")),
            Chain::Broken(why) => Err(format!(
                "its seal cannot be checked through the chain: {why}"
            )),
            Chain::Open { programme, systems } => {
                let before_1_0 = release
                    .meta
                    .get("upgraded_from")
                    .is_some_and(|v| v == vleo_files::upgrade::FROM_FORMAT_1);
                let checked = if before_1_0 {
                    // Checked by the programme manager's anchor of its
                    // fingerprint; no mount is read.
                    chain::check_release_in(release, programme, programme)
                } else {
                    let parents: Vec<&Registry> =
                        std::iter::once(programme).chain(systems.iter()).collect();
                    let (parent, _) = chain::mounted_by(id, &parents).map_err(|e| {
                        format!(
                            "its seal cannot be checked through the chain: {}",
                            e.message()
                        )
                    })?;
                    chain::check_release_in(release, parent, programme)
                }
                .map_err(|e| {
                    format!("its seal does not check through the chain: {}", e.message())
                })?;
                if !checked.holds() {
                    return Err(format!(
                        "its seal does not check through the chain: {}",
                        checked.refused.join("; ")
                    ));
                }
                let how = if before_1_0 {
                    "a release from before 1.0, anchored by the programme manager"
                } else {
                    "checked through the chain"
                };
                Ok(format!(
                    "sealed by {sealed_by}: {how}, from the anchor START HERE names"
                ))
            }
        }
    }
}

impl Taken {
    /// How its seal checked, as said now: whether this computer has accepted
    /// the anchor is read when it is said, not when the design was built, so
    /// a person who accepts it on opening their key sees it accepted.
    pub fn chain_said(&self, drive: &Path) -> String {
        if !self.anchored {
            return self.chain.clone();
        }
        match crate::me::drive_anchor(drive) {
            Ok((_, true)) => self.chain.clone(),
            Ok((_, false)) => format!(
                "{}; this computer has not accepted that anchor yet: open a key on this drive and compare it",
                self.chain
            ),
            Err(why) => format!(
                "{}; whether this computer accepted that anchor cannot be read: {why}",
                self.chain
            ),
        }
    }
}

impl Today {
    /// The fingerprint of today's design: the design's own files it was built
    /// on (`base`, their fingerprint), and every sheet the releases taken
    /// wrote. Two instances on one drive, on the same design, give the same
    /// one (W6); with nothing taken it is the design's own fingerprint, since
    /// today's design is then the design's own files.
    pub fn fingerprint(&self, base: &str) -> String {
        fingerprint(base, &self.written)
    }

    /// Every node a taken release changes in the design, with which release:
    /// what changed since the design's own sheets, for tracing a closure that
    /// held before and does not now (`vleo_modules::health::trace_since`).
    pub fn changed(&self) -> Vec<(String, String)> {
        self.groups
            .iter()
            .filter_map(|g| g.taken.as_ref().map(|t| (g, t)))
            .flat_map(|(g, t)| {
                t.changes
                    .iter()
                    .map(move |n| (n.clone(), format!("changed by {} {}", g.id, t.version)))
            })
            .collect()
    }
}

impl Group {
    /// The group's state in a sentence: which release it is, and whether that
    /// is its latest.
    pub fn said(&self) -> String {
        match (&self.taken, self.refused.first()) {
            (Some(t), None) => format!("{} {} — its latest", self.id, t.version),
            (Some(t), Some((v, why))) => format!(
                "{} {} — its latest, {v}, is refused: {why}",
                self.id, t.version
            ),
            (None, Some((v, why))) => format!(
                "{} — no release passes; the design's own sheets. Its latest, {v}, is refused: {why}",
                self.id
            ),
            (None, None) => format!("{} — no release on the drive", self.id),
        }
    }
}

/// The design `base` holds, with every group's latest release on `drive` that
/// passes taken into it. `root` is the design's root, as the loader reads it.
pub fn build(base: Arc<dyn Files>, root: &Path, drive: &Path) -> Result<Today, String> {
    let groups_dir = drive.join("groups");
    if !groups_dir.is_dir() {
        return Err(format!(
            "{} is not a drive: it has no groups/ folder",
            drive.display()
        ));
    }
    let mut ignored = Vec::new();
    let mut found: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&groups_dir)
        .map_err(|e| format!("{}: {e}", groups_dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.join("releases").is_dir())
        .collect();
    dirs.sort();
    for dir in dirs {
        let id = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut files: Vec<PathBuf> = std::fs::read_dir(dir.join("releases"))
            .map_err(|e| format!("{}: {e}", dir.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_file())
            .collect();
        files.sort();
        for f in files {
            let name = f
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if !name.ends_with(".vleo") {
                ignored.push(format!(
                    "groups/{id}/releases/{name}: not a release — releases/ holds sealed releases only"
                ));
            } else if name.contains(" (") || name.contains("(1)") {
                ignored.push(format!(
                    "groups/{id}/releases/{name}: two files where there should be one — its owner keeps one"
                ));
            } else {
                found.entry(id.clone()).or_default().push(f);
            }
        }
    }

    let chain = chain_of(drive);
    let mut taken: BTreeMap<PathBuf, Vec<u8>> = BTreeMap::new();
    let mut groups = Vec::new();
    for (id, files) in found {
        // Newest first, by the version each release says it is; one that does
        // not open is tried in its place by name, and refused there.
        let mut releases: Vec<Candidate> = files
            .iter()
            .map(|f| {
                let r = Release::open(f);
                let v = r
                    .as_ref()
                    .ok()
                    .map(|r| r.version.clone())
                    .unwrap_or_else(|| {
                        f.file_stem()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or_default()
                    });
                (ReleaseVersion::parse(&v), v, f.clone(), r)
            })
            .collect();
        releases.sort_by(|a, b| b.0.cmp(&a.0));
        let mut group = Group {
            id: id.clone(),
            taken: None,
            refused: Vec::new(),
        };
        for (version, said, path, opened) in releases {
            let tried = opened.and_then(|r| {
                if version.is_none() {
                    return Err(format!("'{said}' is not a release's version"));
                }
                take(&base, &taken, root, &id, &path, &r, &chain).map(|t| (r, t))
            });
            match tried {
                Ok((r, (edits, changes, notes, how))) => {
                    taken.extend(edits);
                    group.taken = Some(Taken {
                        version: r.version,
                        sealed: r.sealed,
                        sealed_by: r.sealed_by,
                        fingerprint: r.fingerprint,
                        changes,
                        notes,
                        anchored: matches!(chain, Chain::Open { .. }),
                        chain: how.said,
                        based_on: how.based_on,
                        base_version: how.base_version,
                    });
                    break;
                }
                Err(why) => group.refused.push((said, why)),
            }
        }
        groups.push(group);
    }
    let written = digest(root, &taken);
    let files: Arc<dyn Files> = Arc::new(Overlay {
        base,
        replaced: taken,
    });
    // The whole design, read again through every check the loader makes.
    vleo_sheet::load::load_all_from(&*files, root)
        .map_err(|e| format!("today's design does not load: {e}"))?;
    Ok(Today {
        files,
        groups,
        ignored,
        written,
        base: base_of(drive),
    })
}

/// Every sheet replaced, by its path under `root` and the hash of its bytes,
/// as one digest; empty when none is. The path is the one under the root, so
/// two copies of the tool in two folders agree.
fn digest(root: &Path, taken: &BTreeMap<PathBuf, Vec<u8>>) -> String {
    if taken.is_empty() {
        return String::new();
    }
    let mut all = Vec::new();
    for (path, bytes) in taken {
        let under = path.strip_prefix(root).unwrap_or(path);
        all.extend_from_slice(under.to_string_lossy().replace('\\', "/").as_bytes());
        all.push(0);
        all.extend_from_slice(&vleo_files::keys::sha256(bytes));
    }
    vleo_files::keys::hex(&vleo_files::keys::sha256(&all))
}

/// The fingerprint of a design built on `base` with `written` taken into it.
pub fn fingerprint(base: &str, written: &str) -> String {
    if written.is_empty() {
        return base.to_string();
    }
    let all = format!("today's design\0{base}\0{written}");
    vleo_files::keys::hex(&vleo_files::keys::sha256(all.as_bytes()))
}

/// A release found on the drive: the version it says it is, as said and as
/// read, where it is, and the release or why it does not open.
type Candidate = (
    Option<ReleaseVersion>,
    String,
    PathBuf,
    Result<Release, String>,
);

/// The sheets one release writes into the design as it stands, the nodes it
/// changes, what a person should know of it, and how its seal checked — or
/// why it is refused.
type Edits = (
    BTreeMap<PathBuf, Vec<u8>>,
    Vec<String>,
    Vec<String>,
    Checked,
);

/// How a release's seal checked, and the design it was based on.
struct Checked {
    said: String,
    based_on: String,
    base_version: Option<u64>,
}

fn take(
    base: &Arc<dyn Files>,
    taken: &BTreeMap<PathBuf, Vec<u8>>,
    root: &Path,
    id: &str,
    path: &Path,
    r: &Release,
    chain: &Chain,
) -> Result<Edits, String> {
    if r.group != id {
        return Err(format!(
            "it says it is {}'s release, in {id}'s folder",
            if r.group.is_empty() {
                "no group"
            } else {
                &r.group
            }
        ));
    }
    r.check_seal()
        .map_err(|e| format!("its files are not the ones sealed: {e}"))?;
    let file = upgraded(path, r)?;
    let how = Checked {
        said: chain.check(id, &file, &r.sealed_by)?,
        based_on: file.meta.get("based_on").cloned().unwrap_or_default(),
        base_version: file.base_version(),
    };
    let findings = vleo_files::checks::check_release(&file);
    if !findings.holds() {
        let errors: Vec<String> = findings
            .0
            .iter()
            .filter(|f| f.level == vleo_files::checks::Level::Error)
            .map(|f| format!("{}: {}", f.place, f.what))
            .collect();
        return Err(errors.join("; "));
    }
    let now = Overlay {
        base: base.clone(),
        replaced: taken.clone(),
    };
    let tree = vleo_sheet::load::load_all_from(&now, root)
        .map_err(|e| format!("the design does not load to take it into: {e}"))?;
    let derisk = r.latest_version();
    let (mut edits, mut changes) = (BTreeMap::new(), Vec::new());
    let mut notes = Vec::new();
    let mut refused = Vec::new();
    for n in r.records("nodes.csv")? {
        let node = n.get("id").cloned().unwrap_or_default();
        if n.get("kind").map(String::as_str) != Some("computed") {
            continue;
        }
        let Some(sh) = tree.sheets.get(&node) else {
            notes.push(format!(
                "{node} is not in the design: a new node joins it through the design's own files"
            ));
            continue;
        };
        let path = sh.dir.join("node.toml");
        let now_text = now
            .read_to_string(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let (form, not_taken) =
            intake::node_form(root, r, sh, vleo_sheet::form::file_hash(&now_text), &derisk)?;
        if let Some(why) = not_taken {
            notes.push(format!(
                "{node}: transcribed, but {why}: taken as a relation an assistant supplied"
            ));
        }
        let plan = vleo_sheet::template::plan_form_against(&tree, form)
            .map_err(|e| format!("{node}: {e}"))?;
        for item in &plan.items {
            if let vleo_sheet::template::Verdict::Conflict(why)
            | vleo_sheet::template::Verdict::Refused(why) = &item.verdict
            {
                refused.push(format!("{node} · {}: {why}", item.what));
            }
        }
        if plan.applicable() > 0 {
            if let Some(text) = plan.text {
                edits.insert(path, text.into_bytes());
                changes.push(node);
            }
        }
    }
    if !refused.is_empty() {
        return Err(refused.join("; "));
    }
    Ok((edits, changes, notes, how))
}

/// A release as the one schema holds it, for the checks of its content: a
/// file from before 1.0 upgraded in memory, with nothing dropped. The file on
/// the drive is not touched.
fn upgraded(path: &Path, r: &Release) -> Result<vleo_files::model::File, String> {
    let how = vleo_files::upgrade::Upgrade {
        app: concat!("vleo ", env!("CARGO_PKG_VERSION")),
        at: &r.sealed,
    };
    vleo_files::sqlite::open(path, &how)
        .map(|o| o.file)
        .map_err(|e| format!("it does not open in the one schema: {e}"))
}

/// The design with some of its sheets replaced: every other file is the
/// base's.
struct Overlay {
    base: Arc<dyn Files>,
    replaced: BTreeMap<PathBuf, Vec<u8>>,
}

impl Files for Overlay {
    fn read(&self, p: &Path) -> std::io::Result<Vec<u8>> {
        match self.replaced.get(p) {
            Some(b) => Ok(b.clone()),
            None => self.base.read(p),
        }
    }
    fn entries(&self, p: &Path) -> std::io::Result<Vec<PathBuf>> {
        self.base.entries(p)
    }
    fn is_dir(&self, p: &Path) -> bool {
        self.base.is_dir(p)
    }
    fn is_file(&self, p: &Path) -> bool {
        self.replaced.contains_key(p) || self.base.is_file(p)
    }
}
