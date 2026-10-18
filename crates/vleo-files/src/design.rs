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
//! **Release design** ([`release`]) is refused whole when any line fails. A
//! released design holds each group's release taken whole, by its bytes and
//! their SHA-256, beside the programme's and the systems' files the same way,
//! and a table of what is in it. Their rows are not merged: a reader opens
//! each as today's design opens a release. It is signed by the system
//! engineer, or their deputy, and held to `chain::check_design` as it will be
//! read.
//!
//! Nothing is computed here: every number is a release's own, and what a
//! closure does with it is the engine's.
//!
//! Not here: the pointer that makes a design current and the archive (F7),
//! previews and their answers (F6-c), the daily snapshot.

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
        // A declaration's findings are the assistant's line; every other
        // finding of the release's checks is the nodes' line.
        let mut declared: Vec<String> = Vec::new();
        for b in t.file.blocks.iter().filter(|b| b.archived == 0) {
            if b.behaviour == "children" {
                continue;
            }
            for e in checks::declared(t.file, b).errors() {
                declared.push(format!("{}: {}", e.place, e.what));
            }
        }
        let found = checks::check_release(t.file);
        let nodes: Vec<String> = found
            .errors()
            .map(|e| format!("{}: {}", e.place, e.what))
            .filter(|e| !declared.contains(e))
            .collect();
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

/// Every port of `f` whose range is declared by one end only, whose ends
/// are not numbers, or whose lower end is above its upper.
fn ranges(f: &File) -> Vec<String> {
    let mut out = Vec::new();
    let id = |uid: &str| {
        f.blocks
            .iter()
            .find(|b| b.uid == uid)
            .map_or(uid.to_string(), |b| b.id.clone())
    };
    for p in &f.ports {
        let (lo, hi) = (p.lower.trim(), p.upper.trim());
        if lo.is_empty() && hi.is_empty() {
            continue;
        }
        let at = format!("{}.{}", id(&p.block_uid), p.name);
        if lo.is_empty() || hi.is_empty() {
            out.push(format!("{at} declares one end of its range only"));
            continue;
        }
        let num = |v: &str| v.parse::<f64>().ok();
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

/// The design `version`, released by `by` with `key` at `at`, from today's
/// design: refused whole unless `checks` holds, `by` is the system engineer
/// or their deputy with a key the programme's file counts then, and
/// `version` comes after `previous`, the design released now. `changed` is
/// what changed, in the system engineer's words; `app` the application
/// releasing it, the oldest that can run it.
#[allow(clippy::too_many_arguments)]
pub fn release(
    checks: &Integration,
    held: &Held,
    version: &str,
    previous: Option<&str>,
    changed: &str,
    by: &str,
    key: &SigningKey,
    at: &str,
    app: &str,
    programme: &Registry,
) -> Result<File, Error> {
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
    let v = DesignVersion::parse(version).ok_or_else(|| {
        refused(format!(
            "{version:?} is not a design's version: year.month.sequence, such as 2026.11.2"
        ))
    })?;
    if let Some(p) = previous {
        let was = DesignVersion::parse(p).ok_or_else(|| {
            refused(format!(
                "the design released now says it is {p:?}, which is not a design's version"
            ))
        })?;
        if v <= was {
            return Err(refused(format!(
                "{version} does not come after {p}, the design released now"
            )));
        }
    }
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
    let mut csv = String::from("group,version,fingerprint,path,sha256\n");
    f.media.push(whole(&held.programme));
    f.media.push(whole(&held.systems));
    for (group, version, fingerprint, w) in &held.releases {
        csv.push_str(&format!(
            "{group},{version},{fingerprint},{},{}\n",
            w.path,
            hex(&sha256(w.bytes))
        ));
        f.media.push(whole(w));
    }
    f.tables.push(Tbl {
        scope: FILE.into(),
        path: "released.csv".into(),
        csv,
    });
    f.texts.push(Text {
        scope: FILE.into(),
        kind: "changed".into(),
        body: changed.trim().to_string(),
    });
    let mut checked = String::new();
    for l in &checks.lines {
        checked.push_str(&format!("{}: {}: {}\n", l.group, l.check.name(), l.says));
    }
    f.texts.push(Text {
        scope: FILE.into(),
        kind: "integration".into(),
        body: checked,
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
