//! An issue, from raised to closed (docs/briefs/F5.md, item 6; W11).
//!
//! An issue file is never changed once it is written: it is its raiser's.
//! What becomes of it is read from the files of the people it is for:
//!
//! - **assigned** by its group's owner, in the group file, a text of the
//!   file's own named `issue <number>` holding the person
//!   (`breakdown::Edit::AssignIssue`);
//! - **resolved** by a release whose notes name it: its de-risking record,
//!   the group's `versions.csv` row for its version, lists it in its
//!   `resolves` column;
//! - **closed** when that release is in today's design and the issue's mark
//!   is no longer red.

use crate::chain::FILE;
use crate::model::File;

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

/// The issues `release`'s notes resolve: the numbers in the `resolves`
/// column of its group's `versions.csv` row for its version, in order,
/// each once. A release with no such row resolves nothing.
pub fn resolves(release: &File) -> Vec<u64> {
    let version = release.meta.get("version").map_or("", String::as_str);
    let Some(t) = release
        .tables
        .iter()
        .find(|t| t.scope == FILE && t.path == "versions.csv")
    else {
        return Vec::new();
    };
    let (head, rows) = crate::csv::parse(&t.csv);
    let col = |name: &str| head.iter().position(|h| crate::csv::name_of(h) == name);
    let (Some(v), Some(r)) = (col("version"), col("resolves")) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for row in rows
        .iter()
        .filter(|row| row.get(v).map(|x| x.trim()) == Some(version))
    {
        for n in row
            .get(r)
            .map_or("", String::as_str)
            .split([',', ';', ' '])
            .filter_map(|x| x.trim().trim_start_matches('#').parse::<u64>().ok())
        {
            if !out.contains(&n) {
                out.push(n);
            }
        }
    }
    out
}

/// What has become of issue `number`: assigned to `assigned`, with
/// `taken`, the release of its group in today's design (its version and
/// the release), and `red`, whether its mark is red there now.
pub fn state(
    number: u64,
    assigned: Option<&str>,
    taken: Option<(&str, &File)>,
    red: bool,
) -> State {
    if let Some((version, release)) = taken {
        if resolves(release).contains(&number) {
            let by = version.to_string();
            return if red {
                State::Resolved { by }
            } else {
                State::Closed { by }
            };
        }
    }
    match assigned {
        Some(who) => State::Assigned(who.to_string()),
        None => State::Open,
    }
}
