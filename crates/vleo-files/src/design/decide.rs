//! The decision on an objection, and what a release asks of the previews
//! (docs/briefs/F6.md, F6-c items 5 and 6; W7; W8, its step 2).
//!
//! An objection is a group's answer *object, because…* to a preview of a
//! release in today's design ([`Objection`]). The system engineer, or their
//! deputy, decides it, with no new kind of file (F6's question 47):
//!
//! - **ask for its next version**: an issue on the releasing group, to its
//!   owner, at the objection's place `objection:<g>-<v>.<a>`, resolved by the
//!   release that names it (`crate::issues`);
//! - **change an allocation**: `breakdown::Edit::Reallocate` in the systems'
//!   file, its why naming the objection (question 48);
//! - **release anyway**: an issue on `programme`, to the programme manager,
//!   holding the objection's words and the answer's digest.
//!
//! [`decisions`] reads what was decided from those files, each held to the
//! chain: an issue counts only signed by the system engineer, or their
//! deputy, who raised it, naming the answer by its digest; a change only in
//! the systems' file, which the chain opened.
//!
//! The release asks it of today's design ([`asked_of`]): every preview on
//! the design released now of a release today's design holds answered by
//! every group it names; every objection decided; none decided *next
//! version* while today's design still holds the release objected to. Each
//! objection is a red mark, `objection:…`, never marked understood: one
//! decided *release anyway* is raised by the programme manager's issue, and
//! one decided by an allocation changed is closed by that change.
//!
//! Nothing is computed here: what moved is the preview's, and what each
//! group said is its answer's.

use super::{Answered, Mark};
use crate::chain::{self, Registry, FILE};
use crate::error::{Error, ErrorKind};
use crate::keys::SigningKey;
use crate::meta::Kind;
use crate::model::{File, Issue, Text};

fn refused(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Malformed, message)
}

/// The place of the objection of `answering` to `group`'s release `version`:
/// where an issue deciding it is raised, and the red mark it is.
pub fn objection_id(group: &str, version: &str, answering: &str) -> String {
    format!("objection:{group}-{version}.{answering}")
}

/// A group's objection to a release in today's design, as its answer on the
/// drive says it: the release, the group objecting, why, and the answer's
/// digest (`chain::digest` of its content), which a decision names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Objection {
    pub group: String,
    pub version: String,
    pub answering: String,
    pub because: String,
    pub answer: String,
}

impl Objection {
    pub fn id(&self) -> String {
        objection_id(&self.group, &self.version, &self.answering)
    }
}

/// What the system engineer decided of an objection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Its next version asked for, by the issue `issue` to the releasing
    /// group's owner.
    NextVersion { issue: i64, by: String },
    /// An allocation changed in the systems' file, by `by` at `at`.
    Reallocated { by: String, at: String },
    /// Released anyway, by the issue `issue` to the programme manager.
    ReleaseAnyway { issue: i64, by: String },
}

impl Decision {
    /// The decision in words, for a page and for `objections.csv`.
    pub fn said(&self) -> String {
        match self {
            Decision::NextVersion { issue, by } => {
                format!("its next version, asked for by {by} in issue {issue}")
            }
            Decision::Reallocated { by, at } => {
                format!("an allocation changed by {by}, {at}")
            }
            Decision::ReleaseAnyway { issue, by } => {
                format!("released anyway, put by {by} to the programme manager in issue {issue}")
            }
        }
    }
}

/// The text of a deciding issue that says, signed, what its issue row says
/// of the decision.
const DECISION: &str = "decision";

fn decided_as(place: &str, group: &str, addressed_to: &str, answer: &str) -> String {
    format!("{place}\n{group}\n{addressed_to}\nanswer {answer}")
}

/// How the system engineer decides an objection by an issue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decide {
    NextVersion,
    ReleaseAnyway,
}

/// The issue `number` deciding `o`, raised by `by` with `key` at `at`: on the
/// releasing group, to its owner as the file among `parents` that mounts it
/// names them, for its next version; or on `programme`, to its programme
/// manager, to release anyway. It holds the objection's words and the
/// answer's digest. Refused for anyone but the system engineer or their
/// deputy, with a key the programme's file counts then.
#[allow(clippy::too_many_arguments)]
pub fn decision_issue(
    o: &Objection,
    decide: Decide,
    number: i64,
    by: &str,
    key: &SigningKey,
    at: &str,
    app: &str,
    parents: &[&Registry],
    programme: &Registry,
) -> Result<File, Error> {
    if crate::checks::is_assistant(by) {
        return Err(refused(format!(
            "{by} is an assistant's name: an objection is decided by a person"
        )));
    }
    programme.key(by, &key.public().to_hex(), at)?;
    if !programme.acts_as(by, "system engineer") {
        return Err(Error::new(
            ErrorKind::Signature,
            format!(
                "{by} may not decide an objection: the system engineer, or their deputy, decides it (W7)"
            ),
        ));
    }
    if o.because.trim().is_empty() || o.answer.is_empty() {
        return Err(refused(format!(
            "{} is not an objection that says why, by an answer named: nothing is decided",
            o.id()
        )));
    }
    let (group, addressed_to, decided) = match decide {
        Decide::NextVersion => {
            let (_, owner) = chain::mounted_by(&o.group, parents)?;
            (
                o.group.clone(),
                owner.to_string(),
                "ask for its next version",
            )
        }
        Decide::ReleaseAnyway => {
            let pm = programme
                .file
                .people
                .iter()
                .find(|p| p.role == "programme manager" && p.deputy_for.is_empty())
                .ok_or_else(|| {
                    refused("the programme's file registers no programme manager to put it to")
                })?;
            ("programme".to_string(), pm.name.clone(), "release anyway")
        }
    };
    let mut f = File::new(Kind::Issue, app);
    for (k, v) in [
        ("number", number.to_string()),
        ("group_id", group.clone()),
        ("writer", by.to_string()),
    ] {
        f.meta.insert(k.to_string(), v);
    }
    // An issue file's signature does not cover its issue row
    // (`chain::digest` leaves the `issue` table out), so what decides is
    // said again in a text it does cover, and read only where the two agree.
    f.texts.push(Text {
        scope: FILE.into(),
        kind: DECISION.into(),
        body: decided_as(&o.id(), &group, &addressed_to, &o.answer),
    });
    f.issues.push(Issue {
        number,
        group_id: group,
        raised_by: by.to_string(),
        at: at.to_string(),
        place: o.id(),
        what: format!(
            "{} objects to {} {}: {} Decided by {by}: {decided}.",
            o.answering,
            o.group,
            o.version,
            o.because.trim()
        ),
        evidence: format!("answer {}", o.answer),
        addressed_to,
        closed_by: String::new(),
    });
    f.check_meta()?;
    let row = chain::sign(&f, FILE, by, key, at, "ok", "raised")?;
    f.signatures.push(row);
    Ok(f)
}

/// What was decided of each of `objections`, read from `issues`, every issue
/// file of the drive and the copy, and from `systems`, the systems' file the
/// chain opened: an issue at the objection's place naming its answer by its
/// digest, signed by the system engineer or their deputy who raised it, on
/// the releasing group to its owner or on `programme` to its programme
/// manager; or a change of the systems' file reallocating, its why naming
/// the objection. The first that holds is the decision; none, undecided.
pub fn decisions(
    objections: &[Objection],
    issues: &[File],
    systems: &File,
    parents: &[&Registry],
    programme: &Registry,
) -> Vec<Option<Decision>> {
    objections
        .iter()
        .map(|o| {
            let id = o.id();
            let by_issue = issues.iter().find_map(|f| {
                let [i] = f.issues.as_slice() else {
                    return None;
                };
                if i.place != id || i.evidence != format!("answer {}", o.answer) {
                    return None;
                }
                let signed = decided_as(&i.place, &i.group_id, &i.addressed_to, &o.answer);
                let says: Vec<&Text> = f
                    .texts
                    .iter()
                    .filter(|t| t.scope == FILE && t.kind == DECISION)
                    .collect();
                if says.len() != 1 || says[0].body != signed {
                    return None;
                }
                let (counts, _) = chain::counted(f, FILE, programme, |name| {
                    name == i.raised_by && programme.acts_as(name, "system engineer")
                });
                if counts.is_empty() {
                    return None;
                }
                if i.group_id == o.group
                    && chain::mounted_by(&o.group, parents)
                        .is_ok_and(|(_, owner)| owner == i.addressed_to)
                {
                    return Some(Decision::NextVersion {
                        issue: i.number,
                        by: i.raised_by.clone(),
                    });
                }
                let to_pm = programme
                    .file
                    .people
                    .iter()
                    .any(|p| p.name == i.addressed_to && p.role == "programme manager");
                (i.group_id == "programme" && to_pm).then(|| Decision::ReleaseAnyway {
                    issue: i.number,
                    by: i.raised_by.clone(),
                })
            });
            by_issue.or_else(|| {
                systems
                    .changes
                    .iter()
                    .find(|c| c.what.starts_with("reallocate:") && c.what.contains(&id))
                    .map(|c| Decision::Reallocated {
                        by: c.who.clone(),
                        at: c.at.clone(),
                    })
            })
        })
        .collect()
}

/// A preview of a release, as it stands for the release: the release, the
/// design it is on, and what each group it names has said on the drive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Standing {
    pub group: String,
    pub version: String,
    pub based_on: String,
    pub answers: Vec<(String, Answered)>,
}

/// An objection and what was decided of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decided {
    pub objection: Objection,
    pub decision: Option<Decision>,
}

/// What the previews ask of the design `version`, released after `previous`
/// from today's design holding `holding` (each group's release, by its group
/// and version): refused, by name, while a preview on `previous` of a
/// release today's design holds is unanswered or answered by a file that
/// does not count, while an objection to one has no decision, or while one
/// decided *next version* is of a release today's design still holds. A
/// preview of another release, or on another design, is stale and asks
/// nothing (question 46). Gives the red marks the objections are, each
/// raised by its programme manager's issue, and the rows of
/// `objections.csv`.
pub fn asked_of(
    previews: &[Standing],
    decided: &[Decided],
    previous: Option<&str>,
    holding: &[(&str, &str)],
) -> Result<(Vec<Mark>, Vec<Vec<String>>), Error> {
    let held = |g: &str, v: &str| holding.iter().any(|(hg, hv)| *hg == g && *hv == v);
    let live: Vec<&Standing> = previews
        .iter()
        .filter(|p| Some(p.based_on.as_str()) == previous && held(&p.group, &p.version))
        .collect();
    let mut wanting = Vec::new();
    for p in &live {
        for (g, a) in &p.answers {
            match a {
                Answered::Unanswered => wanting.push(format!(
                    "{g} has not answered the preview of {} {}",
                    p.group, p.version
                )),
                Answered::Refused(why) => wanting.push(format!(
                    "{g}'s answer to the preview of {} {} does not count: {why}",
                    p.group, p.version
                )),
                Answered::Fine | Answered::Object(_) => {}
            }
        }
    }
    if !wanting.is_empty() {
        return Err(refused(format!(
            "every answer is in before the design is released (W8), and these are not: {}",
            wanting.join("; ")
        )));
    }
    let mut marks = Vec::new();
    let mut rows = Vec::new();
    let mut open = Vec::new();
    for p in &live {
        for (g, a) in &p.answers {
            let Answered::Object(because) = a else {
                continue;
            };
            let id = objection_id(&p.group, &p.version, g);
            let d = decided
                .iter()
                .find(|d| d.objection.id() == id && d.objection.because.trim() == because.trim());
            let Some(Decided {
                objection,
                decision: Some(decision),
            }) = d
            else {
                open.push(format!(
                    "{g} objects to {} {}: {because}",
                    p.group, p.version
                ));
                continue;
            };
            if let Decision::NextVersion { issue, .. } = decision {
                return Err(refused(format!(
                    "{} {} is to be released again, as issue {issue} asks for {g}'s objection: today's design still holds it, so the design is not released until {}'s next version replaces it",
                    p.group, p.version, p.group
                )));
            }
            if let Decision::ReleaseAnyway { issue, .. } = decision {
                marks.push(Mark {
                    id: id.clone(),
                    said: format!(
                        "{g} objects to {} {}: {}; released anyway, for the programme manager",
                        p.group,
                        p.version,
                        because.trim()
                    ),
                    raised: Some(*issue),
                });
            }
            rows.push(vec![
                id,
                p.group.clone(),
                p.version.clone(),
                g.clone(),
                because.trim().to_string(),
                objection.answer.clone(),
                decision.said(),
            ]);
        }
    }
    if !open.is_empty() {
        return Err(refused(format!(
            "every objection is decided before the design is released (W7), and these are not: {}. Ask for its next version, change an allocation, or release anyway",
            open.join("; ")
        )));
    }
    Ok((marks, rows))
}

/// The columns of a released design's `objections.csv`.
pub const OBJECTIONS: [&str; 7] = [
    "objection",
    "group",
    "version",
    "answering",
    "because",
    "answer",
    "decision",
];
