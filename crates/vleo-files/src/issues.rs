//! An issue, from raised to closed (docs/briefs/F5.md, item 6; W11).
//!
//! An issue file is never changed once it is written: it is its raiser's.
//! What becomes of it is read from the files of the people it is for:
//!
//! - **assigned** by its group's owner, in the group file, a text of the
//!   file's own named `issue <number>` holding the person
//!   (`breakdown::Edit::AssignIssue`);
//! - **resolved** by a release whose notes name it: its de-risking record,
//!   the group's `versions.csv`, lists it in the `resolves` column of the
//!   row of that release or of any release before it, since the record is
//!   carried whole from release to release;
//! - **closed** when that release is in today's design and the issue's mark
//!   is no longer red.
//!
//! An issue upgraded from before 1.0 may say who closed it then
//! (`closed_by`): that is read as it was said, and nothing else of its
//! state is.

use crate::chain::FILE;
use crate::meta::ReleaseVersion;
use crate::model::{File, Issue};

/// What has become of an issue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Open,
    /// Assigned to this person, and not yet resolved.
    Assigned(String),
    /// Named by the release taken, of this version, and still red.
    Resolved {
        by: String,
    },
    /// Named by the release taken, of this version, and red no longer.
    Closed {
        by: String,
    },
    /// Closed before 1.0, as its upgraded file says, by whom it says.
    ClosedBefore1_0 {
        by: String,
    },
}

impl State {
    /// The state said, for the group `group` it is addressed to.
    pub fn said(&self, group: &str) -> String {
        match self {
            State::Open => "open".into(),
            State::Assigned(who) => format!("assigned to {who}"),
            State::Resolved { by } => {
                format!("resolved by {group} {by}, and its mark is still red")
            }
            State::Closed { by } => format!("closed by {group} {by}"),
            State::ClosedBefore1_0 { by } => format!("closed by {by}, before 1.0"),
        }
    }
}

/// The text of a group file that holds who an issue is assigned to.
pub fn text_kind(number: u64) -> String {
    format!("issue {number}")
}

/// Who issue `number` is assigned to in the group file `group`, if anyone.
pub fn assigned(group: &File, number: u64) -> Option<String> {
    let kind = text_kind(number);
    group
        .texts
        .iter()
        .find(|t| t.scope == FILE && t.kind == kind)
        .map(|t| t.body.trim().to_string())
        .filter(|w| !w.is_empty())
}

/// The issues `release`'s notes resolve, each with the version whose row
/// names it: the numbers in the `resolves` column of every row of its
/// group's `versions.csv` up to its own version, in order, each once. The
/// record is carried whole from release to release, so a later release
/// still resolves what an earlier one did. A release with no record
/// resolves nothing; an entry that is not an issue's number, or a row whose
/// version is not one, is refused by its row, never skipped.
pub fn resolves(release: &File) -> Result<Vec<(u64, String)>, String> {
    let version = release.meta.get("version").map_or("", String::as_str);
    let upto = ReleaseVersion::parse(version)
        .ok_or_else(|| format!("the release says its version is {version:?}, which is not one"))?;
    let Some(t) = release
        .tables
        .iter()
        .find(|t| t.scope == FILE && t.path == "versions.csv")
    else {
        return Ok(Vec::new());
    };
    let (head, rows) = crate::csv::parse(&t.csv);
    let col = |name: &str| head.iter().position(|h| crate::csv::name_of(h) == name);
    let (Some(v), Some(r)) = (col("version"), col("resolves")) else {
        return Ok(Vec::new());
    };
    let mut out: Vec<(u64, String)> = Vec::new();
    for row in &rows {
        let at = row.get(v).map_or("", |x| x.trim());
        let named = row.get(r).map_or("", |x| x.trim());
        let Some(rv) = ReleaseVersion::parse(at) else {
            if named.is_empty() {
                continue;
            }
            return Err(format!(
                "versions.csv's row {at:?} resolves {named:?}, and {at:?} is not a release's version"
            ));
        };
        if rv > upto {
            continue;
        }
        for x in named
            .split([',', ';', ' '])
            .map(str::trim)
            .filter(|x| !x.is_empty())
        {
            let n = x.trim_start_matches('#').parse::<u64>().map_err(|_| {
                format!("versions.csv's row {at} resolves {x:?}, which is not an issue's number")
            })?;
            if !out.iter().any(|(m, _)| *m == n) {
                out.push((n, at.to_string()));
            }
        }
    }
    Ok(out)
}

/// What has become of `issue`: assigned to `assigned`, with `taken`, its
/// group's release in today's design, and `red`, whether its mark is red
/// there now. A release of another group is refused: issue numbers are not
/// a group's own, so another group's notes never resolve this one.
pub fn state(
    issue: &Issue,
    assigned: Option<&str>,
    taken: Option<&File>,
    red: bool,
) -> Result<State, String> {
    if !issue.closed_by.trim().is_empty() {
        return Ok(State::ClosedBefore1_0 {
            by: issue.closed_by.trim().to_string(),
        });
    }
    if let Some(release) = taken {
        let of = release.meta.get("group_id").map_or("", String::as_str);
        if of != issue.group_id {
            return Err(format!(
                "the release given is {of}'s, and issue {} is addressed to {}",
                issue.number, issue.group_id
            ));
        }
        let number = u64::try_from(issue.number).unwrap_or(u64::MAX);
        if let Some((_, by)) = resolves(release)?.into_iter().find(|(n, _)| *n == number) {
            return Ok(if red {
                State::Resolved { by }
            } else {
                State::Closed { by }
            });
        }
    }
    Ok(match assigned {
        Some(who) => State::Assigned(who.to_string()),
        None => State::Open,
    })
}
