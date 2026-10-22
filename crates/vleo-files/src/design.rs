//! The design released, at the system engineer's valve (docs/briefs/F6.md,
//! F6-b; W8; docs/OPERATING_1_0.md, section 15).
//!
//! **The integration checks** ([`integration`]) are W8's five, for each
//! group, each held or failing with why, read from today's design as it is
//! built: the release it took, or the refusal of its latest.
//!
//! - **the seal and the chain**: a release was taken, its seal checked;
//! - **the base still fitting**: its `based_on` is the design released now;
//! - **every node's method, cases and record**: `checks::check_release`;
//! - **no relation supplied by an assistant**: every node's declaration;
//! - **both ends of every declared range**, each a number, the lower not
//!   above the upper. Why it is that range is the port's `range_reason`,
//!   which the schema keeps optional, so it is not refused here (F6's
//!   question 35).
//!
//! Beside them, not refusing, the red marks ([`Mark`]): each closure that
//! does not close and each issue open, raised as an issue or marked
//! understood before the design is released (question 34).
//!
//! **Release design** ([`release`]) is refused whole when any line fails. A
//! released design holds each group's release taken whole, by its bytes and
//! their SHA-256, beside the programme's and the systems' files the same way,
//! a table of what is in it, and the checks it was released on. Its version
//! follows the design released now: the next in its month, or the first of a
//! later one. Their rows are not merged: a reader opens
//! each as today's design opens a release. It is signed by the system
//! engineer, or their deputy, and held to `chain::check_design` as it will be
//! read.
//!
//! **A released design read back** ([`open_held`], [`released_in`], F6-c):
//! opened through the chain, every file it holds held to its SHA-256 and to
//! its row of `released.csv`, each opening as the release that row names.
//! What relies on a released design reads it so.
//!
//! Nothing is computed here: every number is a release's own, and what a
//! closure does with it is the engine's.
//!
//! **A preview, and the answers to it** (`preview`, F6-c): see its module.
//!
//! Not here: the pointer that makes a design current and the archive (F7),
//! the daily snapshot.

mod preview;

pub use preview::{answers, named, Answered, Move, OpenedPreview};
#[cfg(not(target_arch = "wasm32"))]
pub use preview::{open_preview, preview, to_preview};

use crate::chain::{self, Registry, FILE};
use crate::checks;
use crate::error::{Error, ErrorKind};
use crate::keys::{hex, sha256, SigningKey};
use crate::meta::{DesignVersion, Kind};
use crate::model::{File, Media, Tbl, Text};

fn refused(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Malformed, message)
}

/// One of W8's five checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    Seal,
    Base,
    Nodes,
    Assistant,
    Ranges,
}

impl Check {
    pub const ALL: [Check; 5] = [
        Check::Seal,
        Check::Base,
        Check::Nodes,
        Check::Assistant,
        Check::Ranges,
    ];

    /// The check, as W8 says it.
    pub fn name(self) -> &'static str {
        match self {
            Check::Seal => "the seal and the signature chain",
            Check::Base => "the base still fitting",
            Check::Nodes => "every node's method, cases and record",
            Check::Assistant => "no relation supplied by an assistant",
            Check::Ranges => "both ends of every declared range",
        }
    }
}

/// One line of the checks: a check, for a group, held or failing with why.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub group: String,
    pub check: Check,
    pub holds: bool,
    pub says: String,
}

/// Every line of the integration checks.
#[derive(Clone, Debug, PartialEq)]
pub struct Integration {
    pub lines: Vec<Line>,
}

impl Integration {
    /// Whether every line holds: a design is released only then.
    pub fn holds(&self) -> bool {
        !self.lines.is_empty() && self.lines.iter().all(|l| l.holds)
    }

    /// Every line that fails, as one sentence each.
    pub fn failing(&self) -> Vec<String> {
        self.lines
            .iter()
            .filter(|l| !l.holds)
            .map(|l| format!("{}, {}: {}", l.group, l.check.name(), l.says))
            .collect()
    }
}

/// A group's part of today's design: the release it took, with how its seal
/// checked, or why its latest was refused and none was taken.
pub struct Part<'a> {
    pub group: &'a str,
    pub taken: Result<Taken<'a>, &'a str>,
}

/// A release today's design took.
pub struct Taken<'a> {
    pub version: &'a str,
    /// How its seal checked through the chain, as today's design says it.
    pub chain: &'a str,
    /// The release as the drive holds it, read as the one schema.
    pub file: &'a File,
}

/// The integration checks of today's design: `parts`, one for each group,
/// and `released`, the design released now, or empty before the first.
pub fn integration(parts: &[Part], released: &str) -> Integration {
    let mut lines = Vec::new();
    for p in parts {
        let line = |check: Check, holds: bool, says: String| Line {
            group: p.group.to_string(),
            check,
            holds,
            says,
        };
        let t = match &p.taken {
            Ok(t) => t,
            Err(why) => {
                // Nothing is taken, so nothing else of it can hold.
                lines.push(line(
                    Check::Seal,
                    false,
                    format!("no release of {} is taken: {why}", p.group),
                ));
                for c in &Check::ALL[1..] {
                    lines.push(line(
                        *c,
                        false,
                        format!("not checked: no release of {} is taken", p.group),
                    ));
                }
                continue;
            }
        };
        lines.push(line(
            Check::Seal,
            true,
            format!("{} {} is taken: {}", p.group, t.version, t.chain),
        ));
        let based_on = t.file.meta.get("based_on").map_or("", String::as_str);
        lines.push(if based_on == released {
            line(
                Check::Base,
                true,
                if released.is_empty() {
                    "based on no released design, before the first".to_string()
                } else {
                    format!("based on {released}, the design released now")
                },
            )
        } else {
            line(
                Check::Base,
                false,
                format!(
                    "based on {}, and the design released now is {}: its group moves its base (W12) and seals again",
                    if based_on.is_empty() { "no released design" } else { based_on },
                    if released.is_empty() { "none" } else { released }
                ),
            )
        });
        // The assistant's line: each computed node's declaration, the one
        // `check_release` asks of it, and every signature in an assistant's
        // name. Every other finding of the release's checks is the nodes'
        // line. Both are read from `check_release`'s own findings, so a
        // finding is on one line or the other, never on neither.
        let mut asked: Vec<(String, String)> = Vec::new();
        for b in t
            .file
            .blocks
            .iter()
            .filter(|b| b.archived == 0 && b.behaviour == "method")
        {
            for e in checks::declared(t.file, b).errors() {
                asked.push((e.place.clone(), e.what.clone()));
            }
        }
        let by_assistant = |what: &str| {
            t.file.signatures.iter().any(|s| {
                checks::is_assistant(&s.signer)
                    && what.starts_with(&format!("is signed by {:?}", s.signer))
            })
        };
        let (mut declared, mut nodes) = (Vec::new(), Vec::new());
        for e in checks::check_release(t.file).errors() {
            let said = format!("{}: {}", e.place, e.what);
            if asked.contains(&(e.place.clone(), e.what.clone())) || by_assistant(&e.what) {
                declared.push(said);
            } else {
                nodes.push(said);
            }
        }
        lines.push(line(
            Check::Nodes,
            nodes.is_empty(),
            if nodes.is_empty() {
                "every node's method, cases and record hold".to_string()
            } else {
                nodes.join("; ")
            },
        ));
        lines.push(line(
            Check::Assistant,
            declared.is_empty(),
            if declared.is_empty() {
                "every node says who made its method, and no assistant supplied one".to_string()
            } else {
                declared.join("; ")
            },
        ));
        let ranges = ranges(t.file);
        lines.push(line(
            Check::Ranges,
            ranges.is_empty(),
            if ranges.is_empty() {
                "every declared range has both ends, in order".to_string()
            } else {
                ranges.join("; ")
            },
        ));
    }
    Integration { lines }
}

/// Every port of a live block of `f` whose range is declared by one end
/// only, whose ends are not finite numbers, or whose lower end is above its
/// upper.
fn ranges(f: &File) -> Vec<String> {
    let mut out = Vec::new();
    let id = |uid: &str| {
        f.blocks
            .iter()
            .find(|b| b.uid == uid)
            .map_or(uid.to_string(), |b| b.id.clone())
    };
    let live = |uid: &str| f.blocks.iter().any(|b| b.uid == uid && b.archived == 0);
    for p in f.ports.iter().filter(|p| live(&p.block_uid)) {
        let (lo, hi) = (p.lower.trim(), p.upper.trim());
        if lo.is_empty() && hi.is_empty() {
            continue;
        }
        let at = format!("{}.{}", id(&p.block_uid), p.name);
        if lo.is_empty() || hi.is_empty() {
            out.push(format!("{at} declares one end of its range only"));
            continue;
        }
        let num = |v: &str| v.parse::<f64>().ok().filter(|x| x.is_finite());
        match (num(lo), num(hi)) {
            (Some(a), Some(b)) if a <= b => {}
            (Some(_), Some(_)) => out.push(format!(
                "{at} runs from {lo} to {hi}: its lower end is above its upper"
            )),
            _ => out.push(format!("{at}'s range, {lo} to {hi}, is not two numbers")),
        }
    }
    out
}

/// A file a released design holds whole: where the drive keeps it, and its
/// bytes.
pub struct Whole<'a> {
    pub path: &'a str,
    pub bytes: &'a [u8],
}

/// What the released design holds, and of what.
pub struct Held<'a> {
    /// Each group's release taken, by its group, version and fingerprint.
    pub releases: Vec<(&'a str, &'a str, &'a str, Whole<'a>)>,
    pub programme: Whole<'a>,
    pub systems: Whole<'a>,
}

/// Whether the design `version` may follow `previous`, the design released
/// now (F6's question 39): year.month.sequence, the next in the month of
/// `previous`, or the first of a later month; the first design of all is the
/// first of its month. Why not, when it may not.
pub fn follows(version: &str, previous: Option<&str>) -> Result<(), String> {
    let v = DesignVersion::parse(version).ok_or_else(|| {
        format!("{version:?} is not a design's version: year.month.sequence, such as 2026.11.2")
    })?;
    let Some(p) = previous.filter(|p| !p.is_empty()) else {
        return if v.sequence == 1 {
            Ok(())
        } else {
            Err(format!(
                "{version} is not the first of its month: the first design released is {}.{}.1",
                v.year, v.month
            ))
        };
    };
    let was = DesignVersion::parse(p).ok_or_else(|| {
        format!("the design released now says it is {p:?}, which is not a design's version")
    })?;
    if v <= was {
        return Err(format!(
            "{version} does not come after {p}, the design released now"
        ));
    }
    let next = was.next(v.year, v.month);
    if v != next {
        return Err(format!(
            "{version} does not follow {p}, the design released now: the next in {}.{} is {next}",
            v.year, v.month
        ));
    }
    Ok(())
}

/// A red mark of today's design, beside the checks and not refusing one: a
/// closure that does not close, or an issue open (W8's step 2; F6's
/// question 34). Each is marked understood by the system engineer, or is
/// raised as an issue, and the released design records which.
#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    /// What names it on the page: `closure:<node>`, `issue:<number>`.
    pub id: String,
    pub said: String,
    /// The open issue it is raised as, when it is.
    pub raised: Option<i64>,
}

/// What the system engineer asks for when they release the design:
/// `version`, after `previous`, the design released now; `changed`, in their
/// words; and `understood`, every red mark of `marks` they mark understood.
pub struct Asked<'a> {
    pub version: &'a str,
    pub previous: Option<&'a str>,
    pub changed: &'a str,
    pub marks: &'a [Mark],
    pub understood: &'a [String],
}

/// The design `asked.version`, released by `by` with `key` at `at`, from
/// today's design: refused whole unless `checks` holds, every red mark is
/// raised as an issue or marked understood, `by` is the system engineer or
/// their deputy with a key the programme's file counts then, and the
/// version comes after the design released now. `app` is the application
/// releasing it, the oldest that can run it.
#[allow(clippy::too_many_arguments)]
pub fn release(
    checks: &Integration,
    held: &Held,
    asked: &Asked,
    by: &str,
    key: &SigningKey,
    at: &str,
    app: &str,
    programme: &Registry,
) -> Result<File, Error> {
    let Asked {
        version,
        previous,
        changed,
        marks,
        understood,
    } = *asked;
    if !checks.holds() {
        let failing = checks.failing();
        return Err(refused(if failing.is_empty() {
            "nothing is checked, so nothing is released: today's design holds no group".to_string()
        } else {
            format!(
                "the design is not released, refused whole: {}",
                failing.join("; ")
            )
        }));
    }
    // What is held is what was checked: a release for each group the checks
    // read, and none beside.
    let mut holding: Vec<&str> = held.releases.iter().map(|r| r.0).collect();
    holding.sort_unstable();
    let mut sorted: Vec<&str> = checks.lines.iter().map(|l| l.group.as_str()).collect();
    sorted.sort_unstable();
    sorted.dedup();
    if holding != sorted {
        return Err(refused(format!(
            "the releases held ({}) are not the groups checked ({}): a released design holds what was checked",
            holding.join(", "),
            sorted.join(", ")
        )));
    }
    if let Some(u) = understood
        .iter()
        .find(|u| !marks.iter().any(|m| &m.id == *u))
    {
        return Err(refused(format!(
            "{u} is marked understood, and today's design has no such red mark: look at the red marks again"
        )));
    }
    let open: Vec<&str> = marks
        .iter()
        .filter(|m| m.raised.is_none() && !understood.contains(&m.id))
        .map(|m| m.said.as_str())
        .collect();
    if !open.is_empty() {
        return Err(refused(format!(
            "every red mark is understood before the design is released (W8), and these are not: {}. Mark each understood, or raise it as an issue",
            open.join("; ")
        )));
    }
    if crate::checks::is_assistant(by) {
        return Err(refused(format!(
            "{by} is an assistant's name: a design is released by a person"
        )));
    }
    programme.key(by, &key.public().to_hex(), at)?;
    if !programme.acts_as(by, "system engineer") {
        return Err(Error::new(
            ErrorKind::Signature,
            format!(
                "{by} may not release the design: the system engineer, or their deputy, releases it (W8)"
            ),
        ));
    }
    follows(version, previous).map_err(refused)?;
    if changed.trim().is_empty() {
        return Err(refused(
            "say what changed: a released design says what changed in it, for everyone it reaches",
        ));
    }
    let app_version = app.split_whitespace().nth(1).unwrap_or("");
    let mut f = File::new(Kind::Design, app);
    f.meta.insert("version".into(), version.into());
    f.meta
        .insert("previous".into(), previous.unwrap_or("").into());
    f.meta.insert("released_by".into(), by.into());
    f.meta
        .insert("oldest_application".into(), app_version.to_string());
    let whole = |w: &Whole| Media {
        scope: FILE.into(),
        path: w.path.into(),
        media_type: "application/vnd.vleo".into(),
        sha256: hex(&sha256(w.bytes)),
        bytes: w.bytes.to_vec(),
    };
    let mut rows = Vec::new();
    f.media.push(whole(&held.programme));
    f.media.push(whole(&held.systems));
    for (group, version, fingerprint, w) in &held.releases {
        rows.push(vec![
            group.to_string(),
            version.to_string(),
            fingerprint.to_string(),
            w.path.to_string(),
            hex(&sha256(w.bytes)),
        ]);
        f.media.push(whole(w));
    }
    f.tables.push(Tbl {
        scope: FILE.into(),
        path: "released.csv".into(),
        csv: crate::csv::write(
            &["group", "version", "fingerprint", "path", "sha256"],
            &rows,
        ),
    });
    let red: Vec<Vec<String>> = marks
        .iter()
        .map(|m| {
            vec![
                m.id.clone(),
                m.said.clone(),
                match m.raised {
                    Some(n) => format!("raised as issue {n}"),
                    None => format!("understood by {by}"),
                },
            ]
        })
        .collect();
    f.tables.push(Tbl {
        scope: FILE.into(),
        path: "red_marks.csv".into(),
        csv: crate::csv::write(&["mark", "said", "how"], &red),
    });
    f.texts.push(Text {
        scope: FILE.into(),
        kind: "changed".into(),
        body: changed.trim().to_string(),
    });
    let lines: Vec<Vec<String>> = checks
        .lines
        .iter()
        .map(|l| {
            vec![
                l.group.clone(),
                l.check.name().to_string(),
                if l.holds { "holds" } else { "fails" }.to_string(),
                l.says.clone(),
            ]
        })
        .collect();
    f.tables.push(Tbl {
        scope: FILE.into(),
        path: "integration.csv".into(),
        csv: crate::csv::write(&["group", "check", "holds", "says"], &lines),
    });
    f.check_meta()?;
    let row = chain::sign(&f, FILE, by, key, at, "ok", "")?;
    f.signatures.push(row);
    chain::check_design(&f, programme).map_err(|e| {
        refused(format!(
            "the design would not open through the chain as it will be read, so it is not released: {}",
            e.message()
        ))
    })?;
    Ok(f)
}

/// The released design as CSV anyone can open without the application
/// (docs/OPERATING_1_0.md, section 15, `readable/`): `Groups`, `Nodes`,
/// `Ports`, `Wires`, `Closures` and `Status`, each by its file name, from
/// `design`, the released design, and `releases`, each group's release it
/// holds, by its group, read as the one schema. Every value is a release's
/// own, as it holds it; what the engine makes of them is not here (the
/// application writes `Health` beside these from the engine's map).
pub fn readable(design: &File, releases: &[(&str, &File)]) -> Vec<(&'static str, String)> {
    let get = |f: &File, k: &str| f.meta.get(k).cloned().unwrap_or_default();
    let version = get(design, "version");
    let groups: Vec<Vec<String>> = releases
        .iter()
        .map(|(g, f)| {
            vec![
                g.to_string(),
                get(f, "version"),
                get(f, "sealed_by"),
                get(f, "sealed_at"),
                get(f, "fingerprint"),
                version.clone(),
            ]
        })
        .collect();
    let (mut nodes, mut ports, mut wires, mut closures) = (vec![], vec![], vec![], vec![]);
    for (g, f) in releases {
        let id = |uid: &str| {
            f.blocks
                .iter()
                .find(|b| b.uid == uid)
                .map_or(uid.to_string(), |b| b.id.clone())
        };
        for b in f.blocks.iter().filter(|b| b.archived == 0) {
            nodes.push(vec![
                g.to_string(),
                b.id.clone(),
                id(&b.parent_uid),
                b.behaviour.clone(),
                b.question.clone(),
                b.revision.to_string(),
            ]);
        }
        for p in &f.ports {
            ports.push(vec![
                g.to_string(),
                id(&p.block_uid),
                p.direction.clone(),
                p.name.clone(),
                p.unit.clone(),
                p.value.clone(),
                p.lower.clone(),
                p.upper.clone(),
                p.state.clone(),
            ]);
        }
        for w in &f.wires {
            wires.push(vec![
                g.to_string(),
                id(&w.to_block),
                w.to_port.clone(),
                w.from_ref.clone(),
            ]);
        }
        for c in &f.closures {
            closures.push(vec![
                g.to_string(),
                id(&c.block_uid),
                c.required_ref.clone(),
                c.achieved_ref.clone(),
                c.sense.clone(),
            ]);
        }
    }
    let status: Vec<Vec<String>> = design
        .tables
        .iter()
        .filter(|t| t.scope == FILE && t.path == "integration.csv")
        .flat_map(|t| crate::csv::read(&t.csv).rows)
        .map(|mut r| {
            r.resize(4, String::new());
            r.push(version.clone());
            r
        })
        .collect();
    use crate::csv::write;
    vec![
        (
            "Groups.csv",
            write(
                &[
                    "group",
                    "release",
                    "sealed_by",
                    "sealed_at",
                    "fingerprint",
                    "design",
                ],
                &groups,
            ),
        ),
        (
            "Nodes.csv",
            write(
                &[
                    "group",
                    "node",
                    "parent",
                    "behaviour",
                    "question",
                    "revision",
                ],
                &nodes,
            ),
        ),
        (
            "Ports.csv",
            write(
                &[
                    "group",
                    "node",
                    "direction",
                    "port",
                    "unit",
                    "value",
                    "lower",
                    "upper",
                    "state",
                ],
                &ports,
            ),
        ),
        (
            "Wires.csv",
            write(&["group", "node", "port", "from"], &wires),
        ),
        (
            "Closures.csv",
            write(
                &["group", "node", "required", "achieved", "sense"],
                &closures,
            ),
        ),
        (
            "Status.csv",
            write(&["group", "check", "holds", "says", "design"], &status),
        ),
    ]
}

/// The latest released design in `dir`, the drive's or the copy's `design/`,
/// by the version each `design-<version>.vleo` is named for; none when it
/// holds none. Each counts only when it is the released design it is named
/// for and reads back through `programme`'s chain ([`open_held`]): a file
/// planted under a design's name is refused by name, never taken for one,
/// and never passed over.
#[cfg(not(target_arch = "wasm32"))]
pub fn latest_in(dir: &std::path::Path, programme: &Registry) -> Result<Option<String>, Error> {
    let entries = match std::fs::read_dir(dir) {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(Error::new(ErrorKind::Io, format!("{}: {e}", dir.display()))),
    };
    let mut latest: Option<(DesignVersion, String)> = None;
    for e in entries {
        let e = e.map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", dir.display())))?;
        let name = e.file_name().to_string_lossy().into_owned();
        let Some(v) = name
            .strip_prefix("design-")
            .and_then(|n| n.strip_suffix(".vleo"))
        else {
            continue;
        };
        let parsed = DesignVersion::parse(v).ok_or_else(|| {
            refused(format!(
                "design/{name} is not named for a design's version: year.month.sequence"
            ))
        })?;
        let f = crate::sqlite::read(&e.path())
            .map_err(|x| refused(format!("design/{name} does not read: {}", x.message())))?;
        if f.meta.get("version").map(String::as_str) != Some(v) {
            return Err(refused(format!(
                "design/{name} is not design {v}: it says it is {:?}",
                f.meta.get("version").map_or("", String::as_str)
            )));
        }
        open_held(&f, programme, concat!("vleo ", env!("CARGO_PKG_VERSION"))).map_err(|x| {
            refused(format!(
                "design/{name} is not a released design that opens through the chain: {}",
                x.message()
            ))
        })?;
        if latest.as_ref().is_none_or(|(l, _)| parsed > *l) {
            latest = Some((parsed, v.to_string()));
        }
    }
    Ok(latest.map(|(_, v)| v))
}

/// A file a released design holds, read back: where the drive keeps it, its
/// bytes, and the file they read as in the one schema.
#[derive(Clone, Debug)]
pub struct Back {
    pub path: String,
    pub bytes: Vec<u8>,
    pub file: File,
}

/// A group's release a released design holds, read back.
#[derive(Clone, Debug)]
pub struct HeldRelease {
    pub group: String,
    pub version: String,
    pub fingerprint: String,
    pub back: Back,
}

/// A released design read back (F6-c, `open_held`): the design, and each
/// file it holds, every one checked.
#[derive(Clone, Debug)]
pub struct Opened {
    pub design: File,
    pub version: String,
    pub releases: Vec<HeldRelease>,
    pub programme: Back,
    pub systems: Back,
}

/// `group`'s, in words: the programme's, the systems', `l3_solar`'s.
#[cfg(not(target_arch = "wasm32"))]
fn whose(group: &str) -> String {
    match group {
        "programme" => "the programme's".into(),
        "systems" => "the systems'".into(),
        g => format!("{g}'s"),
    }
}

/// The paths a released design holds the programme's and the systems' files
/// at.
pub const PROGRAMME_AT: &str = "groups/programme/programme.vgroup";
pub const SYSTEMS_AT: &str = "groups/systems/systems.vgroup";

/// What `bytes` read as in the one schema, a release from before 1.0
/// upgraded as it is read (`sqlite::open`), through a file of its own in the
/// system's temporary folder, gone once read.
#[cfg(not(target_arch = "wasm32"))]
fn read_bytes(bytes: &[u8], how: &crate::upgrade::Upgrade) -> Result<File, Error> {
    use std::io::Write as _;
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let at = std::env::temp_dir().join(format!(
        "vleo-held-{}-{nanos}-{}.vleo",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let io = |e: std::io::Error| Error::new(ErrorKind::Io, format!("{}: {e}", at.display()));
    let mut out = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&at)
        .map_err(io)?;
    let wrote = out.write_all(bytes).map_err(io);
    drop(out);
    let read = wrote.and_then(|()| crate::sqlite::open(&at, how).map(|o| o.file));
    let _ = std::fs::remove_file(&at);
    read
}

/// The released design `f` read back, as it is read wherever one is relied
/// on (F6-c; docs/briefs/F6.md, item 1): it opens through `programme`'s
/// chain (`chain::check_design`), and every file it holds is the one it
/// says: its bytes give the SHA-256 kept beside them, each release is a row
/// of `released.csv` and each row a file held, once, and each opens as the
/// group's release, version and fingerprint its row names; the programme's
/// and the systems' files are held, and are those files. Anything else is
/// refused by name, never passed over.
#[cfg(not(target_arch = "wasm32"))]
pub fn open_held(f: &File, programme: &Registry, app: &str) -> Result<Opened, Error> {
    let version = f.meta.get("version").cloned().unwrap_or_default();
    let named = |what: String| refused(format!("design {version} {what}"));
    chain::check_design(f, programme)?;
    let at = f
        .signatures
        .iter()
        .find(|s| s.scope == FILE)
        .map_or(String::new(), |s| s.signed_at.clone());
    let how = crate::upgrade::Upgrade { app, at: &at };
    let mut seen: Vec<&str> = Vec::new();
    for m in &f.media {
        if m.scope != FILE {
            return Err(named(format!(
                "holds {} for {:?}: a released design holds its files whole, for itself",
                m.path, m.scope
            )));
        }
        if seen.contains(&m.path.as_str()) {
            return Err(named(format!("holds {} twice", m.path)));
        }
        seen.push(&m.path);
        if hex(&sha256(&m.bytes)) != m.sha256 {
            return Err(named(format!(
                "holds {} whose bytes are not the ones it was released with: their SHA-256 differs",
                m.path
            )));
        }
    }
    let media = |path: &str| f.media.iter().find(|m| m.path == path);
    let back = |path: &str, group: &str| -> Result<Back, Error> {
        let m = media(path).ok_or_else(|| named(format!("does not hold {path}")))?;
        let file = read_bytes(&m.bytes, &how).map_err(|e| {
            named(format!(
                "holds {path}, which does not read: {}",
                e.message()
            ))
        })?;
        if file.meta.get("group_id").map(String::as_str) != Some(group) {
            return Err(named(format!(
                "holds {path}, which is not {} file: it says it is {:?}'s",
                whose(group),
                file.meta.get("group_id").map_or("", String::as_str)
            )));
        }
        Ok(Back {
            path: path.to_string(),
            bytes: m.bytes.clone(),
            file,
        })
    };
    let programme_back = back(PROGRAMME_AT, "programme")?;
    let systems_back = back(SYSTEMS_AT, "systems")?;
    for b in [&programme_back, &systems_back] {
        if b.file.kind().ok() != Some(Kind::Group) {
            return Err(named(format!(
                "holds {}, which is not a group's file",
                b.path
            )));
        }
    }
    let table = f
        .tables
        .iter()
        .find(|t| t.scope == FILE && t.path == "released.csv")
        .ok_or_else(|| named("has no released.csv: what it holds is not said".into()))?;
    let csv = crate::csv::read(&table.csv);
    if !csv.problems.is_empty() {
        return Err(named(format!(
            "says what it holds in a released.csv that does not read: {}",
            csv.problems
                .iter()
                .map(|(l, p)| format!("line {l}: {p}"))
                .collect::<Vec<_>>()
                .join("; ")
        )));
    }
    let col = |name: &str| csv.head.iter().position(|h| h == name);
    let (Some(g), Some(v), Some(fp), Some(p), Some(sha)) = (
        col("group"),
        col("version"),
        col("fingerprint"),
        col("path"),
        col("sha256"),
    ) else {
        return Err(named(
            "says what it holds in a released.csv without its columns: group, version, fingerprint, path, sha256".into(),
        ));
    };
    let mut releases: Vec<HeldRelease> = Vec::new();
    for row in &csv.rows {
        let (group, rv, fingerprint, path) = (&row[g], &row[v], &row[fp], &row[p]);
        if releases.iter().any(|r| &r.group == group) {
            return Err(named(format!("holds two releases of {group}")));
        }
        if path == PROGRAMME_AT || path == SYSTEMS_AT {
            return Err(named(format!(
                "names {path} as a group's release: it is the {} file",
                if path == PROGRAMME_AT {
                    "programme's"
                } else {
                    "systems'"
                }
            )));
        }
        let in_releases = path
            .strip_prefix(&format!("groups/{group}/releases/"))
            .is_some_and(|n| {
                n.len() > ".vleo".len()
                    && n.ends_with(".vleo")
                    && !n.contains(['/', '\\'])
                    && !n.starts_with('.')
            });
        if !crate::breakdown::valid_id(group) || !in_releases {
            return Err(named(format!(
                "names {path:?} as {group}'s release: a group's release is held where the drive keeps it, groups/{group}/releases/<name>.vleo"
            )));
        }
        let held = back(path, group)?;
        if media(path).map(|m| m.sha256.as_str()) != Some(row[sha].as_str()) {
            return Err(named(format!(
                "holds {path} whose SHA-256 is not the one released.csv names"
            )));
        }
        let said = |k: &str| held.file.meta.get(k).map_or("", String::as_str);
        if held.file.kind().ok() != Some(Kind::GroupRelease) {
            return Err(named(format!(
                "holds {path}, which is not a group's release"
            )));
        }
        if fingerprint.is_empty() {
            return Err(named(format!("holds {path}, which says no fingerprint")));
        }
        if said("version") != rv || said("fingerprint") != fingerprint {
            return Err(named(format!(
                "holds {path} as {group} {rv}, and it is {group} {} with another fingerprint",
                said("version")
            )));
        }
        // The fingerprint is not covered by the seal, so it is worked out
        // again: a release of the one schema gives its own digest; one from
        // before 1.0 was held to its folder's when it was upgraded, which
        // happens only when it says one.
        let before_1_0 = said("upgraded_from") == crate::upgrade::FROM_FORMAT_1;
        if !before_1_0 && chain::digest(&held.file, FILE)? != *fingerprint {
            return Err(named(format!(
                "holds {path}, whose fingerprint is not its content's: it is not the release that was sealed"
            )));
        }
        releases.push(HeldRelease {
            group: group.clone(),
            version: rv.clone(),
            fingerprint: fingerprint.clone(),
            back: held,
        });
    }
    if let Some(extra) = f.media.iter().find(|m| {
        m.path != PROGRAMME_AT
            && m.path != SYSTEMS_AT
            && !releases.iter().any(|r| r.back.path == m.path)
    }) {
        return Err(named(format!(
            "holds {}, which released.csv does not name",
            extra.path
        )));
    }
    Ok(Opened {
        design: f.clone(),
        version,
        releases,
        programme: programme_back,
        systems: systems_back,
    })
}

/// Every released design in `dirs` (the drive's `design/`, and its archive
/// once there is one), each read back through `open_held`, oldest first: a
/// file under a design's name that does not is refused by name.
#[cfg(not(target_arch = "wasm32"))]
pub fn released_in(
    dirs: &[&std::path::Path],
    programme: &Registry,
    app: &str,
) -> Result<Vec<Opened>, Error> {
    let mut out: Vec<(DesignVersion, Opened)> = Vec::new();
    for dir in dirs {
        let entries = match std::fs::read_dir(dir) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(Error::new(ErrorKind::Io, format!("{}: {e}", dir.display()))),
        };
        let mut names: Vec<String> = Vec::new();
        for e in entries {
            let e = e.map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", dir.display())))?;
            names.push(e.file_name().to_string_lossy().into_owned());
        }
        names.sort();
        for name in names {
            let Some(v) = name
                .strip_prefix("design-")
                .and_then(|n| n.strip_suffix(".vleo"))
            else {
                continue;
            };
            let parsed = DesignVersion::parse(v).ok_or_else(|| {
                refused(format!(
                    "{}/{name} is not named for a design's version: year.month.sequence",
                    dir.display()
                ))
            })?;
            if out.iter().any(|(p, _)| *p == parsed) {
                return Err(refused(format!(
                    "design {v} is held twice, the second in {}",
                    dir.display()
                )));
            }
            let f = crate::sqlite::read(&dir.join(&name))
                .map_err(|x| refused(format!("{name} does not read: {}", x.message())))?;
            if f.meta.get("version").map(String::as_str) != Some(v) {
                return Err(refused(format!(
                    "{name} is not design {v}: it says it is {:?}",
                    f.meta.get("version").map_or("", String::as_str)
                )));
            }
            let opened = open_held(&f, programme, app)
                .map_err(|x| refused(format!("{}/{name}: {}", dir.display(), x.message())))?;
            out.push((parsed, opened));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out.into_iter().map(|(_, o)| o).collect())
}
