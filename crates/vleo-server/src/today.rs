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
//! release from before 1.0 found there is taken only when it passes, in this
//! order (one in 1.0's one schema, as said below):
//!
//! 1. it opens, as a sealed group release (`vleo_files::intake::Release`);
//! 2. it says it is the group whose folder it is in;
//! 3. its files are the ones sealed — every file's SHA-256 gives the
//!    fingerprint it was sealed with (`vleo_files::seal`);
//! 4. its content holds: every check today's intake makes of a release
//!    (`vleo_files::checks::check_release`), each error named;
//! 5. each of its computed nodes, as the change its node engineer would have
//!    made (`vleo_sheet::template::Form`), plans against the design without a
//!    conflict or a refusal
//!    (`vleo_sheet::template`) — a method or results an assistant supplied, a
//!    change with no de-risking record, a case that is not in SI.
//!
//! A release that fails any of them is refused, by its reason, and the group's
//! next older release is tried. A group with none that passes keeps the
//! design's own sheets for its nodes, and says so. Nothing on the drive is
//! written: the releases are applied to the design in memory, sheet by sheet,
//! and the tree is read again through every check the loader makes.
//!
//! And each release's seal is checked through the chain from the anchor its
//! START HERE names (`vleo_files::chain`, docs/briefs/F2.md): the programme's
//! file opened under that anchor, and each release from before 1.0 by the
//! programme manager's anchor of its fingerprint.
//!
//! A release in 1.0's one schema, sealed by its group's owner
//! (`vleo_files::release`, docs/briefs/F5.md), is taken only through the
//! chain: its seal checked through the systems' file that mounts its group,
//! every node in it through the group file it carries, and every check of a
//! release holding. Each node its group assigns then replaces the design's
//! own file of that node, read as the loader reads it
//! (`vleo_files::convert::node_folder`); its headings stay the design's own. A seal that does not check is refused like any other reason,
//! and the group's last good release taken in its place. Whether this
//! computer has accepted the anchor the seals were checked from is said beside
//! each release taken, read when it is said (`Taken::chain_said`). A drive that
//! names an anchor and has no programme's file admits nothing; only a drive
//! with neither, and no anchor accepted for it, is built unchecked, and says
//! so for every release. The design's base is read from the systems' file
//! once it checks through the chain.

use std::collections::{BTreeMap, BTreeSet};
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
/// today's design is based on. It is read only once it checks through the
/// chain; a drive with no chain says its base is not checked.
fn base_of(drive: &Path, chain: &Chain) -> Base {
    let none = |said: String| Base {
        said,
        ..Base::default()
    };
    let path = drive.join("groups/systems/systems.vgroup");
    if !path.is_file() {
        return none(
            "the drive has no systems' file, so no released design is named as its base".into(),
        );
    }
    let f = match vleo_files::sqlite::read(&path) {
        Ok(f) => f,
        Err(e) => {
            return none(format!(
                "the systems' file does not read, so its base is not known: {}",
                e.message()
            ))
        }
    };
    if f.meta.get("group_id").map(String::as_str) != Some("systems") {
        return none(
            "the file in groups/systems is not the systems' file, so its base is not known".into(),
        );
    }
    let unchecked = match chain {
        Chain::Open { programme, .. } => {
            if let Err(e) = chain::open_systems(&f, programme) {
                return none(format!(
                    "the systems' file does not check through the chain, so its base is not known: {}",
                    e.message()
                ));
            }
            ""
        }
        Chain::Broken(why) => return none(format!("its base is not known: {why}")),
        Chain::Absent(_) => "; not checked through the chain: the drive has no programme's file",
    };
    let based_on = f.meta.get("based_on").cloned().unwrap_or_default();
    let base_version = f.base_version();
    let kept = f.meta.get("base_version").filter(|v| !v.trim().is_empty());
    let said = match (based_on.is_empty(), base_version, kept) {
        (_, None, Some(v)) => format!("the systems' file says its base_version is {v:?}, which is not a whole number"),
        (true, Some(n), _) if n > 0 => format!(
            "the systems' file counts {n} released designs before it and names none, so its base is not known"
        ),
        (false, Some(0), _) => format!(
            "the systems' file names the released design {based_on} and counts none before it, so its base is not known"
        ),
        (true, _, _) => "based on no released design: none has been released yet".to_string(),
        (false, Some(n), _) => format!("based on the released design {based_on}, the {}", nth(n)),
        (false, None, None) => format!(
            "based on the released design {based_on}; its count is not kept in the systems' file"
        ),
    };
    Base {
        based_on,
        base_version,
        said: format!("{said}{unchecked}"),
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
    /// The anchor it was checked from, when it was: the programme manager's
    /// key fingerprint START HERE named when the design was built.
    pub anchor: Option<String>,
    /// The released design it was based on, by name and by count
    /// (`base_version`); empty and none for a release from before they were
    /// kept.
    pub based_on: String,
    pub base_version: Option<u64>,
    /// Where the drive holds it.
    pub path: PathBuf,
}

/// The chain the drive's releases are checked through. One is made for a
/// build, so its largest variant's size is no cost worth a box.
#[allow(clippy::large_enum_variant)]
enum Chain {
    /// The drive has no programme's file and names no anchor, and this
    /// computer has accepted none for it: nothing is checked, and each
    /// release says why.
    Absent(String),
    /// The drive's chain does not open: no release is taken.
    Broken(String),
    /// The programme's file, opened under `anchor`.
    Open { programme: Registry, anchor: String },
}

/// The chain `drive` holds, read once for a build.
fn chain_of(drive: &Path) -> Chain {
    let file = drive.join("groups/programme/programme.vgroup");
    let named = crate::me::drive_anchor(drive);
    if !file.is_file() {
        // A drive that names an anchor, or that this computer accepted one
        // for, has a chain: its programme's file missing is the chain broken,
        // never a drive with nothing to check.
        let accepted = crate::me::accepted_anchor(drive);
        if named.is_ok() || !matches!(accepted, Ok(None)) {
            return Chain::Broken(
                "the drive has an anchor and no programme's file, so no seal can be checked from it"
                    .into(),
            );
        }
        return Chain::Absent(
            "the drive has no programme's file, so there is no anchor to check them from".into(),
        );
    }
    let anchor = match named {
        Ok((a, _)) => a,
        Err(why) => return Chain::Broken(why),
    };
    match vleo_files::sqlite::read(&file)
        .map_err(|e| e.message().to_string())
        .and_then(|f| chain::open_programme(f, &anchor).map_err(|e| e.message().to_string()))
    {
        Ok(programme) => Chain::Open { programme, anchor },
        Err(why) => Chain::Broken(format!(
            "the programme's file does not open under the anchor START HERE names: {why}"
        )),
    }
}

impl Chain {
    /// How `release` checks through the chain: what to say of it when it
    /// does, or why it is refused.
    fn check(&self, release: &vleo_files::model::File, sealed_by: &str) -> Result<String, String> {
        match self {
            Chain::Absent(why) => Ok(format!("its seal is not checked through the chain: {why}")),
            Chain::Broken(why) => Err(format!(
                "its seal cannot be checked through the chain: {why}"
            )),
            Chain::Open { programme, .. } => {
                // A release in the one schema is checked by `check_one`, never
                // here; one that did reach here is refused by name rather than
                // checked by a path nothing exercises.
                let before_1_0 = release
                    .meta
                    .get("upgraded_from")
                    .is_some_and(|v| v == vleo_files::upgrade::FROM_FORMAT_1);
                if !before_1_0 {
                    return Err(
                        "it is a release of 1.0's one schema, which today's design does not take yet"
                            .into(),
                    );
                }
                // Checked by the programme manager's anchor of its
                // fingerprint; no mount is read.
                let checked =
                    chain::check_release_in(release, programme, programme).map_err(|e| {
                        format!("its seal does not check through the chain: {}", e.message())
                    })?;
                if !checked.holds() {
                    return Err(format!(
                        "its seal does not check through the chain: {}",
                        checked.refused.join("; ")
                    ));
                }
                let how = "a release from before 1.0, anchored by the programme manager";
                Ok(format!(
                    "sealed by {sealed_by}: {how}, from the anchor START HERE names"
                ))
            }
        }
    }
}

impl Chain {
    /// How `release`, in 1.0's one schema, checks through the chain: its seal
    /// through the file on `drive` that mounts the group `id`, and every node
    /// it holds through the group file it carries; or why it is refused. A
    /// release of the one schema is checked through the chain or not taken.
    fn check_one(
        &self,
        drive: &Path,
        release: &vleo_files::model::File,
        id: &str,
    ) -> Result<(String, (String, String, String)), String> {
        // A release upgraded from before 1.0 is checked by its anchor alone,
        // which covers neither its cases nor its assignments: in the one
        // schema, only a release sealed in the application is taken.
        if release.meta.contains_key("upgraded_from") {
            return Err(
                "it is a release from before 1.0 written in 1.0's one schema: a release of the one schema is taken only as sealed in the application"
                    .into(),
            );
        }
        // Its fingerprint is not under its seal: it is worked out again from
        // what it holds, or anyone who can write the drive could give one
        // release another's name.
        let fingerprint = chain::digest(release, chain::FILE)
            .map_err(|e| format!("its fingerprint cannot be read: {}", e.message()))?;
        match release.meta.get("fingerprint").map(|f| f.trim()) {
            None | Some("") => {
                return Err(
                    "it says no fingerprint: a release is known by the fingerprint of what it holds"
                        .into(),
                )
            }
            Some(f) if f != fingerprint => {
                return Err(format!(
                    "its fingerprint is {f}, and what it holds gives {fingerprint}: a release is never relabelled"
                ))
            }
            Some(_) => {}
        }
        let programme = match self {
            Chain::Absent(why) => {
                return Err(format!(
                    "it is a release of 1.0's one schema, taken only once its seal checks through the chain, and {why}"
                ))
            }
            Chain::Broken(why) => {
                return Err(format!(
                    "its seal cannot be checked through the chain: {why}"
                ))
            }
            Chain::Open { programme, .. } => programme,
        };
        let path = drive.join("groups/systems/systems.vgroup");
        if !path.is_file() {
            return Err(
                "its seal cannot be checked: the drive has no systems' file, which mounts every group"
                    .into(),
            );
        }
        let systems = vleo_files::sqlite::read(&path)
            .map_err(|e| e.message().to_string())
            .and_then(|f| chain::open_systems(&f, programme).map_err(|e| e.message().to_string()))
            .map_err(|why| {
                format!("its seal cannot be checked: the systems' file does not check through the chain: {why}")
            })?;
        let (parent, _) = chain::mounted_by(id, &[programme, &systems])
            .map_err(|e| format!("its seal does not check through the chain: {}", e.message()))?;
        let checked = chain::check_release_in(release, parent, programme)
            .map_err(|e| format!("its seal does not check through the chain: {}", e.message()))?;
        if !checked.holds() {
            return Err(format!(
                "its seal does not check through the chain: {}",
                checked.refused.join("; ")
            ));
        }
        // Who sealed it, and when, as the seal's own signature says.
        let seal = chain::seal_of(release, parent, programme)
            .map_err(|e| format!("its seal does not check through the chain: {}", e.message()))?;
        Ok((
            format!(
                "sealed by {}, its owner as {} mounts it, from the anchor START HERE names",
                seal.signer, parent.from
            ),
            (seal.signer.clone(), seal.signed_at.clone(), fingerprint),
        ))
    }
}

impl Taken {
    /// How its seal checked, as said now: whether this computer has accepted
    /// the anchor is read when it is said, not when the design was built, so
    /// a person who accepts it on opening their key sees it accepted.
    pub fn chain_said(&self, drive: &Path) -> String {
        let Some(anchor) = &self.anchor else {
            return self.chain.clone();
        };
        match crate::me::accepted_anchor(drive) {
            Ok(Some(accepted)) if accepted == *anchor => self.chain.clone(),
            Ok(Some(_)) => format!(
                "{}; this computer accepted another anchor for this drive than the one it was checked from: start the tool again, so the design is built from the anchor accepted",
                self.chain
            ),
            Ok(None) => format!(
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
    let mut gone: BTreeSet<PathBuf> = BTreeSet::new();
    let mut groups = Vec::new();
    for (id, files) in found {
        // Newest first, by the version each release says it is; one that does
        // not open is tried in its place by name, and refused there.
        let mut releases: Vec<Candidate> = files
            .iter()
            .map(|f| {
                let r = Found::open(f);
                let v = r.as_ref().ok().map(|r| r.said().0).unwrap_or_else(|| {
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
            // Once one is taken, an older release is simply older; but a file
            // that does not open, or says no version, sorts last and would
            // never be tried, so it is said as refused wherever it sorts.
            if group.taken.is_some() {
                match (&opened, &version) {
                    (Err(why), _) => group.refused.push((said, why.clone())),
                    (Ok(_), None) => group
                        .refused
                        .push((said.clone(), format!("'{said}' is not a release's version"))),
                    _ => {}
                }
                continue;
            }
            let tried = opened.and_then(|r| {
                if version.is_none() {
                    return Err(format!("'{said}' is not a release's version"));
                }
                let edits = match &r {
                    Found::Before1_0(r) => take(&base, &taken, &gone, root, &id, &path, r, &chain),
                    Found::OneSchema(f) => {
                        take_one(&base, &taken, &gone, root, &id, f, &chain, drive)
                    }
                };
                edits.map(|t| (r.said(), t))
            });
            match tried {
                Ok((said, (edits, changes, notes, how, removed))) => {
                    let (version, sealed, sealed_by, fingerprint) = match &how.seal {
                        Some((by, at, fp)) => (said.0, at.clone(), by.clone(), fp.clone()),
                        None => said,
                    };
                    for p in &edits {
                        gone.remove(p.0);
                    }
                    taken.extend(edits);
                    for p in removed {
                        taken.remove(&p);
                        gone.insert(p);
                    }
                    group.taken = Some(Taken {
                        version,
                        sealed,
                        sealed_by,
                        fingerprint,
                        changes,
                        notes,
                        anchor: match &chain {
                            Chain::Open { anchor, .. } => Some(anchor.clone()),
                            _ => None,
                        },
                        chain: how.said,
                        based_on: how.based_on,
                        base_version: how.base_version,
                        path: path.clone(),
                    });
                }
                Err(why) => group.refused.push((said, why)),
            }
        }
        groups.push(group);
    }
    let written = digest(root, &taken, &gone)?;
    let files: Arc<dyn Files> = Arc::new(Overlay {
        base,
        replaced: taken,
        removed: gone,
    });
    // The whole design, read again through every check the loader makes.
    vleo_sheet::load::load_all_from(&*files, root)
        .map_err(|e| format!("today's design does not load: {e}"))?;
    Ok(Today {
        files,
        groups,
        ignored,
        written,
        base: base_of(drive, &chain),
    })
}

/// The design `base` holds with exactly the releases the released design
/// `design` holds taken into it, each as its bytes (F6-c; docs/briefs/F6.md,
/// item 2): the released design as the engine reads it. It is read back
/// first through the chain `drive` holds now (`design::open_held`), which
/// holds each file to what the design says; a drive with no chain open
/// builds none. Each release, in its group's order as a build takes them, is
/// opened from a file of its own (`of_bytes`) and taken by the checks a
/// build makes; one that does not pass is refused by name, and nothing is
/// taken in its place.
pub fn build_held(
    base: Arc<dyn Files>,
    root: &Path,
    drive: &Path,
    design: &vleo_files::model::File,
) -> Result<Today, String> {
    build_held_with(base, root, drive, design, None)
}

/// One group's release taken into a released design in place of the one it
/// holds of that group, or beside them where it holds none (F6-c, a
/// preview): its group, version and fingerprint, where the drive keeps it,
/// and its bytes.
pub struct One<'a> {
    pub group: &'a str,
    pub version: &'a str,
    pub fingerprint: &'a str,
    pub path: &'a str,
    pub bytes: &'a [u8],
}

/// The released design `design` built as [`build_held`] builds it, with
/// `one`, when given, taken in place of the release it holds of that group,
/// by the same checks: the design released now with one release in it, as a
/// preview reads it.
pub fn build_held_with(
    base: Arc<dyn Files>,
    root: &Path,
    drive: &Path,
    design: &vleo_files::model::File,
    one: Option<&One>,
) -> Result<Today, String> {
    let chain = chain_of(drive);
    let programme = match &chain {
        Chain::Open { programme, .. } => programme,
        Chain::Absent(why) | Chain::Broken(why) => {
            return Err(format!(
                "a released design is built only through the chain: {why}"
            ))
        }
    };
    let held = vleo_files::design::open_held(
        design,
        programme,
        concat!("vleo ", env!("CARGO_PKG_VERSION")),
    )
    .map_err(|e| e.message().to_string())?;
    let mut releases: Vec<One> = held
        .releases
        .iter()
        .filter(|r| one.is_none_or(|o| o.group != r.group))
        .map(|r| One {
            group: &r.group,
            version: &r.version,
            fingerprint: &r.fingerprint,
            path: &r.back.path,
            bytes: &r.back.bytes,
        })
        .collect();
    if let Some(o) = one {
        releases.push(One { ..*o });
    }
    releases.sort_by(|a, b| a.group.cmp(b.group));
    let mut taken: BTreeMap<PathBuf, Vec<u8>> = BTreeMap::new();
    let mut gone: BTreeSet<PathBuf> = BTreeSet::new();
    let mut groups = Vec::new();
    for r in releases {
        let id = r.group;
        let refused = |why: String| {
            format!(
                "design {} holds {id} {}, which is not taken: {why}",
                held.version, r.version
            )
        };
        let (said, (edits, changes, notes, how, removed)) = of_bytes(r.bytes, |at| {
            let found = Found::open(at)?;
            let edits = match &found {
                Found::Before1_0(rel) => take(&base, &taken, &gone, root, id, at, rel, &chain),
                Found::OneSchema(f) => take_one(&base, &taken, &gone, root, id, f, &chain, drive),
            }?;
            Ok((found.said(), edits))
        })
        .map_err(refused)?;
        // Who sealed it and when are its seal's, as a build takes them.
        let (version, sealed, sealed_by, fingerprint) = match &how.seal {
            Some((by, at, fp)) => (said.0, at.clone(), by.clone(), fp.clone()),
            None => said,
        };
        if version != r.version || fingerprint != r.fingerprint {
            return Err(refused(format!(
                "its bytes say they are {id} {version}, not the release the design names"
            )));
        }
        for p in &edits {
            gone.remove(p.0);
        }
        taken.extend(edits);
        for p in removed {
            taken.remove(&p);
            gone.insert(p);
        }
        groups.push(Group {
            id: id.to_string(),
            taken: Some(Taken {
                version,
                sealed,
                sealed_by,
                fingerprint,
                changes,
                notes,
                anchor: match &chain {
                    Chain::Open { anchor, .. } => Some(anchor.clone()),
                    _ => None,
                },
                chain: how.said,
                based_on: how.based_on,
                base_version: how.base_version,
                path: drive.join(r.path),
            }),
            refused: Vec::new(),
        });
    }
    let written = digest(root, &taken, &gone)?;
    let files: Arc<dyn Files> = Arc::new(Overlay {
        base,
        replaced: taken,
        removed: gone,
    });
    vleo_sheet::load::load_all_from(&*files, root)
        .map_err(|e| format!("design {} does not load: {e}", held.version))?;
    Ok(Today {
        files,
        groups,
        ignored: Vec::new(),
        written,
        // What it is based on is what the systems' file it holds names.
        base: Base {
            based_on: held
                .systems
                .file
                .meta
                .get("based_on")
                .cloned()
                .unwrap_or_default(),
            base_version: held.systems.file.base_version(),
            said: format!(
                "the released design {}, as it holds its releases",
                held.version
            ),
        },
    })
}

/// A release today's design took, read again from `path` on `drive` as the
/// group `id`'s and checked through the chain as a build checks it, for a
/// released design to hold (F6-b): its bytes, the release they read as in
/// the one schema, and how its seal checked, every check made of the bytes
/// held.
pub fn read_again(
    drive: &Path,
    id: &str,
    path: &Path,
) -> Result<(Vec<u8>, vleo_files::model::File, String), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    // Every check is of the bytes read, opened from a file of their own: a
    // file changed on the drive between two reads is never what is held.
    let (file, said) = of_bytes(&bytes, |at| {
        let chain = chain_of(drive);
        match Found::open(at)? {
            Found::OneSchema(f) => {
                if f.meta.get("group_id").map(String::as_str) != Some(id) {
                    return Err(format!("it is not {id}'s release"));
                }
                let (said, _) = chain.check_one(drive, &f, id)?;
                Ok((*f, said))
            }
            Found::Before1_0(r) => {
                if r.group != id {
                    return Err(format!("it is not {id}'s release"));
                }
                r.check_seal()
                    .map_err(|e| format!("its files are not the ones sealed: {e}"))?;
                let f = upgraded(at, &r)?;
                if f.meta.get("fingerprint") != Some(&r.fingerprint) {
                    return Err("its fingerprint is not the one it was sealed with".into());
                }
                let said = chain.check(&f, &r.sealed_by)?;
                Ok((f, said))
            }
        }
    })?;
    Ok((bytes, file, said))
}

/// What `read` makes of `bytes`, written whole to a file of this program's
/// own, outside the drive and the copy, and read there; the file is gone
/// after.
pub fn of_bytes<T>(
    bytes: &[u8],
    read: impl FnOnce(&Path) -> Result<T, String>,
) -> Result<T, String> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "vleo-held-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let at = dir.join("held.vleo");
    let out = std::fs::write(&at, bytes)
        .map_err(|e| format!("{}: {e}", at.display()))
        .and_then(|()| read(&at));
    let _ = std::fs::remove_dir_all(&dir);
    out
}

/// Every sheet replaced, by its path under `root` and the hash of its bytes,
/// as one digest; empty when none is. The path is the one under the root, so
/// two copies of the tool in two folders agree.
fn digest(
    root: &Path,
    taken: &BTreeMap<PathBuf, Vec<u8>>,
    gone: &BTreeSet<PathBuf>,
) -> Result<String, String> {
    if taken.is_empty() && gone.is_empty() {
        return Ok(String::new());
    }
    let mut all = Vec::new();
    for path in gone {
        let under = path.strip_prefix(root).map_err(|_| {
            format!(
                "{} is not under the design's root, so today's design has no fingerprint",
                path.display()
            )
        })?;
        all.extend_from_slice(under.to_string_lossy().replace('\\', "/").as_bytes());
        all.extend_from_slice(b"\0removed\0");
    }
    for (path, bytes) in taken {
        // A sheet outside the root would be named by where this copy of the
        // tool sits, and two copies would disagree on the fingerprint.
        let under = path.strip_prefix(root).map_err(|_| {
            format!(
                "{} is not under the design's root, so today's design has no fingerprint",
                path.display()
            )
        })?;
        all.extend_from_slice(under.to_string_lossy().replace('\\', "/").as_bytes());
        all.push(0);
        all.extend_from_slice(&vleo_files::keys::sha256(bytes));
    }
    Ok(vleo_files::keys::hex(&vleo_files::keys::sha256(&all)))
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
    Result<Found, String>,
);

/// A release as it was found: from before 1.0, read as its folder, or in
/// 1.0's one schema, sealed by its group's owner (F5).
enum Found {
    Before1_0(Release),
    OneSchema(Box<vleo_files::model::File>),
}

impl Found {
    /// The release at `path`: from before 1.0 when it is format 1, else in
    /// the one schema when it is a group's release there, else why not.
    fn open(path: &Path) -> Result<Found, String> {
        match Release::open(path) {
            Ok(r) => Ok(Found::Before1_0(r)),
            Err(old) => match vleo_files::sqlite::read(path) {
                Ok(f) if f.kind().ok() == Some(vleo_files::meta::Kind::GroupRelease) => {
                    Ok(Found::OneSchema(Box::new(f)))
                }
                Ok(f) => Err(format!(
                    "it is a {}, and releases/ holds a group's releases only",
                    f.kind()
                        .map_or("file of no kind".to_string(), |k| k.to_string())
                )),
                // A file in the one schema says why it does not read as one;
                // any other says why it is not a release from before 1.0.
                Err(e) if old.contains("not format 1") => {
                    Err(format!("it does not read as a release: {}", e.message()))
                }
                Err(_) => Err(old),
            },
        }
    }

    /// What it says of itself: its version, when it was sealed, by whom, and
    /// its fingerprint.
    fn said(&self) -> (String, String, String, String) {
        match self {
            Found::Before1_0(r) => (
                r.version.clone(),
                r.sealed.clone(),
                r.sealed_by.clone(),
                r.fingerprint.clone(),
            ),
            Found::OneSchema(f) => {
                let get = |k: &str| f.meta.get(k).cloned().unwrap_or_default();
                (
                    get("version"),
                    get("sealed_at"),
                    get("sealed_by"),
                    get("fingerprint"),
                )
            }
        }
    }
}

/// The sheets one release writes into the design as it stands, the nodes it
/// changes, what a person should know of it, and how its seal checked — or
/// why it is refused.
type Edits = (
    BTreeMap<PathBuf, Vec<u8>>,
    Vec<String>,
    Vec<String>,
    Checked,
    BTreeSet<PathBuf>,
);

/// How a release's seal checked, and the design it was based on.
struct Checked {
    said: String,
    based_on: String,
    base_version: Option<u64>,
    /// Who sealed it, when, and its fingerprint, as its seal's signature and
    /// its own digest say, where the chain read them; none for a release
    /// whose seal is not read through a signature.
    seal: Option<(String, String, String)>,
}

#[allow(clippy::too_many_arguments)]
fn take(
    base: &Arc<dyn Files>,
    taken: &BTreeMap<PathBuf, Vec<u8>>,
    gone: &BTreeSet<PathBuf>,
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
    // The file the chain checks and the release whose content is taken are two
    // reads of one path: they must be the same release, or a file changed
    // between them would have its signed copy checked and its other bytes
    // taken.
    if file.meta.get("fingerprint") != Some(&r.fingerprint)
        || file.meta.get("group_id") != Some(&r.group)
        || file.meta.get("version") != Some(&r.version)
    {
        return Err(
            "it changed while it was read: read again, it is not the release that was checked"
                .into(),
        );
    }
    let how = Checked {
        said: chain.check(&file, &r.sealed_by)?,
        based_on: file.meta.get("based_on").cloned().unwrap_or_default(),
        base_version: file.base_version(),
        seal: None,
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
        removed: gone.clone(),
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
    Ok((edits, changes, notes, how, BTreeSet::new()))
}

/// What a release in 1.0's one schema writes into the design as it stands:
/// taken only when its seal checks through the chain from the file that
/// mounts its group, and every check of a release holds. Each node its
/// group assigns, as the release holds it, replaces the design's own file of
/// that node; a block no assignment names is a heading of the group, and is
/// the design's own still.
#[allow(clippy::too_many_arguments)]
fn take_one(
    base: &Arc<dyn Files>,
    taken: &BTreeMap<PathBuf, Vec<u8>>,
    gone: &BTreeSet<PathBuf>,
    root: &Path,
    id: &str,
    file: &vleo_files::model::File,
    chain: &Chain,
    drive: &Path,
) -> Result<Edits, String> {
    let get = |k: &str| file.meta.get(k).map_or("", String::as_str);
    if get("group_id") != id {
        return Err(format!(
            "it says it is {}'s release, in {id}'s folder",
            if get("group_id").is_empty() {
                "no group"
            } else {
                get("group_id")
            }
        ));
    }
    let (said, seal) = chain.check_one(drive, file, id)?;
    let how = Checked {
        said,
        based_on: get("based_on").to_string(),
        base_version: file.base_version(),
        seal: Some(seal),
    };
    let findings = vleo_files::checks::check_release(file);
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
        removed: gone.clone(),
    };
    let tree = vleo_sheet::load::load_all_from(&now, root)
        .map_err(|e| format!("the design does not load to take it into: {e}"))?;
    let design = design_nodes(root, id)?;
    let (mut edits, mut changes) = (BTreeMap::new(), Vec::new());
    let mut notes = Vec::new();
    let mut removed = BTreeSet::new();
    for b in file.blocks.iter().filter(|b| b.archived == 0) {
        // Unassigned, it is a heading: the release's check holds every other
        // block to an assignment (`chain::check_release_in`). A heading is
        // the design's own still.
        if !file.assignments.iter().any(|a| a.block_uid == b.uid) {
            continue;
        }
        let own = design
            .iter()
            .find(|n| n.blocks.first().is_some_and(|x| x.uid == b.uid));
        let own = own.filter(|o| tree.sheets.contains_key(&o.blocks[0].id));
        let Some(own) = own else {
            notes.push(format!(
                "{} is not in the design: a new node joins it through the design's own files",
                b.id
            ));
            continue;
        };
        if own.blocks[0].id != b.id {
            return Err(format!(
                "it names the design's {} {}: a node is renamed through its group's breakdown, not by a release",
                own.blocks[0].id, b.id
            ));
        }
        let node = vleo_files::release::node_in(file, &b.uid, own);
        let folder = vleo_files::convert::node_folder(root, &node)
            .map_err(|e| format!("{}: {}", b.id, e.message()))?;
        let mut moved = false;
        // What the node's folder no longer holds, the design's own kept but
        // nobody signed: its cases, its parity grid, its lesson.
        if let Some(dir) = folder.first().and_then(|(p, _)| p.parent()) {
            for name in ["fixtures.toml", "parity.csv", vleo_sheet::lesson::FILE] {
                let p = dir.join(name);
                if now.is_file(&p) && !folder.iter().any(|(q, _)| *q == p) {
                    removed.insert(p);
                    moved = true;
                }
            }
        }
        for (path, bytes) in folder {
            if now.read(&path).ok().as_deref() != Some(&bytes[..]) {
                edits.insert(path, bytes);
                moved = true;
            }
        }
        if moved {
            changes.push(b.id.clone());
        }
    }
    // The design with the release in it, read through every check the loader
    // makes: a release the design does not load with is refused, and the
    // group's last good one tried.
    let mut with = now.replaced.clone();
    with.extend(edits.iter().map(|(p, b)| (p.clone(), b.clone())));
    let mut without = now.removed.clone();
    without.extend(removed.iter().cloned());
    for p in edits.keys() {
        without.remove(p);
    }
    let after = Overlay {
        base: base.clone(),
        replaced: with,
        removed: without,
    };
    vleo_sheet::load::load_all_from(&after, root)
        .map_err(|e| format!("the design does not load with it: {e}"))?;
    Ok((edits, changes, notes, how, removed))
}

/// The design's own node files of the group `id`, from the folder the design
/// is read from: `VLEO_DESIGN`, or `design/` under `root`.
pub(crate) fn design_nodes(root: &Path, id: &str) -> Result<Vec<vleo_files::model::File>, String> {
    let dir = crate::design_dir(root)
        .join("groups")
        .join(id)
        .join("nodes");
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(format!("the design's {}: {e}", dir.display())),
    };
    for e in entries.filter_map(Result::ok) {
        let p = e.path();
        if p.extension().is_some_and(|x| x == "vnode") {
            out.push(
                vleo_files::sqlite::read(&p)
                    .map_err(|e| format!("the design's {}: {}", p.display(), e.message()))?,
            );
        }
    }
    Ok(out)
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
pub(crate) struct Overlay {
    pub(crate) base: Arc<dyn Files>,
    pub(crate) replaced: BTreeMap<PathBuf, Vec<u8>>,
    /// Files of the base a release's node no longer holds.
    pub(crate) removed: BTreeSet<PathBuf>,
}

impl Files for Overlay {
    fn read(&self, p: &Path) -> std::io::Result<Vec<u8>> {
        if self.removed.contains(p) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("{} is not in today's design", p.display()),
            ));
        }
        match self.replaced.get(p) {
            Some(b) => Ok(b.clone()),
            None => self.base.read(p),
        }
    }
    fn entries(&self, p: &Path) -> std::io::Result<Vec<PathBuf>> {
        let mut out: Vec<PathBuf> = self
            .base
            .entries(p)?
            .into_iter()
            .filter(|e| !self.removed.contains(e))
            .collect();
        for added in self.replaced.keys().filter(|k| k.parent() == Some(p)) {
            if !out.contains(added) {
                out.push(added.clone());
            }
        }
        out.sort();
        Ok(out)
    }
    fn is_dir(&self, p: &Path) -> bool {
        self.base.is_dir(p)
    }
    fn is_file(&self, p: &Path) -> bool {
        !self.removed.contains(p) && (self.replaced.contains_key(p) || self.base.is_file(p))
    }
}

/// Today's view of a branch (docs/briefs/F5.md, F5-d): today's design with
/// every node of the branch whose latest revision its node engineer signed
/// as it is taken in, each in the design's own file of it, as a release's
/// node is. What its group would seal, seen inside the design before it is.
pub struct View {
    /// The design, as the loader reads it, with the branch's signed nodes.
    pub files: Arc<dyn Files>,
    /// The nodes taken in that change the design, by id.
    pub changes: Vec<String>,
    /// The branch's nodes not taken, and why.
    pub left_out: Vec<(String, String)>,
    /// What the view writes over today's design, as one digest
    /// ([`Today::written`]'s kind): two views that write the same are one.
    pub written: String,
}

/// Today's view of the branch `group`: `today`, today's design, with each of
/// `nodes` (the branch's node files) whose signature counts through
/// `registry`, its group file as the chain opens it.
pub fn branch_view(
    today: Arc<dyn Files>,
    root: &Path,
    group: &str,
    registry: &Registry,
    nodes: &[vleo_files::model::File],
) -> Result<View, String> {
    let now = Overlay {
        base: today.clone(),
        replaced: BTreeMap::new(),
        removed: BTreeSet::new(),
    };
    let tree = vleo_sheet::load::load_all_from(&now, root)
        .map_err(|e| format!("today's design does not load: {e}"))?;
    let design = design_nodes(root, group)?;
    let (mut edits, mut changes, mut left_out) = (BTreeMap::new(), Vec::new(), Vec::new());
    let mut removed = BTreeSet::new();
    for n in nodes {
        let Some(b) = n.blocks.first() else {
            continue;
        };
        if let Err(e) = chain::check_node(n, &b.uid, registry) {
            left_out.push((
                b.id.clone(),
                format!("not signed as it is: {}", e.message()),
            ));
            continue;
        }
        let own = design
            .iter()
            .find(|d| d.blocks.first().is_some_and(|x| x.uid == b.uid))
            .filter(|_| tree.sheets.contains_key(&b.id));
        let Some(own) = own else {
            left_out.push((
                b.id.clone(),
                "a new node, not in the design: it joins today's design through its group's release"
                    .into(),
            ));
            continue;
        };
        let node = vleo_files::release::node_in(n, &b.uid, own);
        let folder = match vleo_files::convert::node_folder(root, &node) {
            Ok(f) => f,
            Err(e) => {
                left_out.push((
                    b.id.clone(),
                    format!("it is not read as the design reads a node: {}", e.message()),
                ));
                continue;
            }
        };
        let mut moved = false;
        // What the node's folder no longer holds is not served, as a
        // release's is not (`take_one`).
        if let Some(dir) = folder.first().and_then(|(p, _)| p.parent()) {
            for name in ["fixtures.toml", "parity.csv", vleo_sheet::lesson::FILE] {
                let p = dir.join(name);
                if now.is_file(&p) && !folder.iter().any(|(q, _)| *q == p) {
                    removed.insert(p);
                    moved = true;
                }
            }
        }
        for (path, bytes) in folder {
            if now.read(&path).ok().as_deref() != Some(&bytes[..]) {
                edits.insert(path, bytes);
                moved = true;
            }
        }
        if moved {
            changes.push(b.id.clone());
        }
    }
    let written = digest(root, &edits, &removed)?;
    let files: Arc<dyn Files> = Arc::new(Overlay {
        base: today,
        replaced: edits,
        removed,
    });
    vleo_sheet::load::load_all_from(&*files, root)
        .map_err(|e| format!("today's view of {group} does not load: {e}"))?;
    Ok(View {
        files,
        changes,
        left_out,
        written,
    })
}

/// A value outside a branch that the branch moves: the node, the group it
/// is in, and its answer in today's design and in today's view of the
/// branch, `None` where it refuses or does not run.
#[derive(Clone, Debug, PartialEq)]
pub struct Moved {
    pub id: String,
    pub group: String,
    pub before: Option<f64>,
    pub after: Option<f64>,
    /// Why it refused, where it refused: its node's own reason.
    pub before_why: Option<String>,
    pub after_why: Option<String>,
}

/// One design run whole: how many values it answered, and every node it
/// blocked, each with why. Never only the values: a run is "n ran, m
/// blocked".
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ran {
    pub ran: usize,
    pub blocked: Vec<(String, String)>,
}

/// The impact of a branch: every value outside it that moves, and how each
/// design ran, the branch's own nodes among them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Impact {
    pub moved: Vec<Moved>,
    pub today: Ran,
    pub view: Ran,
}

/// Whether two answers of a value are the same: equal as numbers (F6's
/// question 44, and its answer (b) of 22 October), so -0.0 and 0.0 are; two
/// NaNs only by the same bits. No threshold is set here, outside the kernel,
/// on what counts as the same.
fn same_answer(a: f64, b: f64) -> bool {
    a == b || a.to_bits() == b.to_bits()
}

/// Every value of a node not `inside` the branch that differs between
/// `today` and `view`, each design run whole on `case`: the impact of the
/// branch on the rest of the design. A value is a node's answer, `<node>`,
/// or another of its outputs, `<node>.<port>`, and is the node's. A value
/// that runs in one and refuses in the other has moved too, with its node's
/// reason. Its group is the nearest group it hangs under, or `the design`
/// at the root. Beside them, how each design ran: what it answered and what
/// it blocked, inside the branch or not, so a value refused in both, or a
/// node of the branch that starts refusing, is said too.
pub fn impact(
    today: &'static vleo_modules::Graph,
    view: &'static vleo_modules::Graph,
    case: &vleo_bus::Case,
    inside: &dyn Fn(&str) -> bool,
) -> Result<Impact, String> {
    let answers =
        |g: &'static vleo_modules::Graph| -> Result<(BTreeMap<String, f64>, Ran), String> {
            let mut all = case.clone();
            all.mode = vleo_bus::RunMode::All;
            all.target = g.nodes.first().map_or("", |n| n.id).to_string();
            let r = g
                .evaluate(&all, &mut vleo_modules::Scratch::for_graph(g))
                .map_err(|f| format!("the design did not run on this case: {f}"))?;
            let ran = Ran {
                ran: r.manifest.ran,
                blocked: r.blocked.into_iter().map(|b| (b.id, b.message)).collect(),
            };
            Ok((r.values.into_iter().map(|v| (v.id, v.value)).collect(), ran))
        };
    let ((was, ran_today), (now, ran_view)) = (answers(today)?, answers(view)?);
    let why = |ran: &Ran, node: &str| {
        ran.blocked
            .iter()
            .find(|(id, _)| id == node)
            .map(|(_, w)| w.clone())
    };
    // The group a node is in: the nearest of its parents that is a group.
    let group_of = |id: &str| -> String {
        let mut at = id.to_string();
        for _ in 0..64 {
            let Some(k) = view.find(&at).or_else(|| today.find(&at)) else {
                break;
            };
            let g = if view.find(&at).is_some() {
                view
            } else {
                today
            };
            let parent = g.nodes[k as usize].parent;
            if parent.is_empty() {
                return "the design".to_string();
            }
            if g.groups.iter().any(|x| x.id == parent) {
                return parent.to_string();
            }
            at = parent.to_string();
        }
        String::new()
    };
    let ids: std::collections::BTreeSet<&String> = was.keys().chain(now.keys()).collect();
    let mut out = Vec::new();
    for id in ids {
        let node = id.split('.').next().unwrap_or(id);
        if inside(node) {
            continue;
        }
        let (b, a) = (was.get(id).copied(), now.get(id).copied());
        let moved = match (b, a) {
            (Some(x), Some(y)) => !same_answer(x, y),
            (None, None) => false,
            _ => true,
        };
        if moved {
            out.push(Moved {
                id: id.clone(),
                group: group_of(node),
                before: b,
                after: a,
                before_why: b.is_none().then(|| why(&ran_today, node)).flatten(),
                after_why: a.is_none().then(|| why(&ran_view, node)).flatten(),
            });
        }
    }
    Ok(Impact {
        moved: out,
        today: ran_today,
        view: ran_view,
    })
}

#[cfg(test)]
mod tests {
    use super::same_answer;

    #[test]
    fn an_answer_moves_when_it_differs_as_a_number() {
        assert!(same_answer(0.0, -0.0));
        assert!(same_answer(1.5, 1.5));
        assert!(!same_answer(1.0, 1.0 + f64::EPSILON));
        assert!(!same_answer(1.0, f64::NAN));
        assert!(same_answer(f64::NAN, f64::NAN));
        assert!(!same_answer(f64::NAN, -f64::NAN));
    }
}
