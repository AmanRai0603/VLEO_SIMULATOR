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
//! What this does not yet check: the signatures through the chain from the
//! programme manager's anchor (`vleo_files::chain`). The drive has no
//! programme release to anchor them until the switch-over (docs/PLAN_1_0.md,
//! phases G and H), and each release says so rather than being taken as if it
//! had been checked.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
}

/// One group, and the release its part of today's design is.
pub struct Group {
    pub id: String,
    /// The release taken, when one passes.
    pub taken: Option<Taken>,
    /// Every newer release refused, newest first, with why.
    pub refused: Vec<(String, String)>,
}

/// The release a group's part of today's design is.
pub struct Taken {
    pub version: String,
    pub sealed: String,
    pub sealed_by: String,
    pub fingerprint: String,
    /// The nodes it changes in the design.
    pub changes: Vec<String>,
    /// What a person should know about it: nodes not in the design, a
    /// transcription taken as an assistant's relation, its signatures not yet
    /// checked through the chain.
    pub notes: Vec<String>,
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
                take(&base, &taken, root, &id, &path, &r).map(|t| (r, t))
            });
            match tried {
                Ok((r, (edits, changes, notes))) => {
                    taken.extend(edits);
                    group.taken = Some(Taken {
                        version: r.version,
                        sealed: r.sealed,
                        sealed_by: r.sealed_by,
                        fingerprint: r.fingerprint,
                        changes,
                        notes,
                    });
                    break;
                }
                Err(why) => group.refused.push((said, why)),
            }
        }
        groups.push(group);
    }
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
    })
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
/// changes, and what a person should know of it — or why it is refused.
type Edits = (BTreeMap<PathBuf, Vec<u8>>, Vec<String>, Vec<String>);

fn take(
    base: &Arc<dyn Files>,
    taken: &BTreeMap<PathBuf, Vec<u8>>,
    root: &Path,
    id: &str,
    path: &Path,
    r: &Release,
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
    let findings = vleo_files::checks::check_release(&upgraded(path, r)?);
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
    let mut notes = vec![
        "its signatures are not yet checked through the chain: the drive has no programme release to anchor them"
            .to_string(),
    ];
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
    Ok((edits, changes, notes))
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
