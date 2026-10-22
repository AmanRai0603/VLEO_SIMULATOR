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
//! - **release anyway**: an issue on `programme`, to the programme manager,
//!   holding the objection's words and the answer's digest;
//! - **change an allocation** (question 48, F6-e): `breakdown::Edit::
//!   Reallocate`, in the copy's systems' file, its why naming the objection
//!   and its answer as [`reallocation_names`] says them, and that file put
//!   back to the drive.
//!
//! [`decisions`] reads what was decided from the issue files, each held to
//! the chain: an issue counts only signed by the system engineer, or their
//! deputy, who raised it, naming the answer by its digest, and saying what
//! it decides again in a text its signature covers. It reads a reallocation
//! from the drive's systems' file, as it opens through the chain, marked so
//! its change rows are signed with it (F6-d): a `reallocate` row made by the
//! system engineer or their deputy, naming the objection and its answer.
//! What it gives ([`Decided`]) only it makes, so a release is given no
//! decision it did not read.
//!
//! The release asks it of today's design ([`asked_of`]): every preview on
//! the design released now of a release today's design holds answered by
//! every group it names; every objection decided; none decided *next
//! version* while today's design still holds the release objected to. One
//! decided *release anyway* is a red mark, `objection:…`, never marked
//! understood, raised by the programme manager's issue; one decided by an
//! allocation changed is not, since what it objected to has changed. A
//! preview that does not open, and may be one the release must hear, holds
//! it as one unanswered does.
//!
//! Nothing is computed here: what moved is the preview's, and what each
//! group said is its answer's.

use super::{Answered, Mark};
use crate::chain::{self, Registry, FILE};
use crate::error::{Error, ErrorKind};
use crate::keys::SigningKey;
use crate::meta::Kind;
use crate::model::{Change, File, Issue, Text};

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
    /// Released anyway, by the issue `issue` to the programme manager.
    ReleaseAnyway { issue: i64, by: String },
    /// An allocation changed, by the `reallocate` change row of the drive's
    /// systems' file that names it, made by `by` at `at`.
    Reallocated { by: String, at: String },
}

impl Decision {
    /// The decision in words, for a page and for `objections.csv`.
    pub fn said(&self) -> String {
        match self {
            Decision::NextVersion { issue, by } => {
                format!("its next version, asked for by {by} in issue {issue}")
            }
            Decision::ReleaseAnyway { issue, by } => {
                format!("released anyway, put by {by} to the programme manager in issue {issue}")
            }
            Decision::Reallocated { by, at } => {
                format!("an allocation changed, by {by} at {at}, in the systems' file")
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

/// What the why of a reallocation says to decide `o`: the objection's place
/// and its answer, by the first [`NAMED`] characters of its digest, so a
/// reallocation made before the group answered again decides nothing of the
/// answer it gives now. A reallocation naming more of the digest names it
/// too.
pub fn reallocation_names(o: &Objection) -> String {
    let named = o.answer.get(..NAMED).unwrap_or(&o.answer);
    format!("{} answer {named}", o.id())
}

/// How much of an answer's digest a reallocation names it by.
const NAMED: usize = 12;

/// Whether the change row `c` is a reallocation deciding `o`: its what is a
/// `reallocate` edit's, and says the objection's place, then `answer` and as
/// much of its answer's digest as [`reallocation_names`] gives, or more.
fn reallocates(c: &Change, o: &Objection) -> bool {
    let Some(why) = c.what.strip_prefix("reallocate:") else {
        return false;
    };
    if o.answer.len() < NAMED {
        return false;
    }
    let place = format!("{} answer ", o.id());
    why.match_indices(&place).any(|(i, _)| {
        let rest = &why[i + place.len()..];
        let hex = rest
            .find(|ch: char| !ch.is_ascii_hexdigit())
            .map_or(rest, |end| &rest[..end]);
        hex.len() >= NAMED && o.answer.starts_with(hex)
    })
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
    // What decides is said again in a text the signature covers, and read
    // only where the two agree: an issue file signed before F6-d's
    // `signed_rows` marker leaves its issue row out of its signature, and is
    // still read.
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
    // Its issue and change rows are signed with it (F6-d).
    chain::sign_rows(&mut f);
    let row = chain::sign(&f, FILE, by, key, at, "ok", "raised")?;
    f.signatures.push(row);
    Ok(f)
}

/// What was decided of each of `objections`, read from `issues`, every issue
/// file of the drive and the copy: an issue file of one issue, its number and
/// group as its signed meta says them, at the objection's place, naming its
/// answer by its digest, saying so in its one signed `decision` text as its
/// row does, signed by the system engineer or their deputy who raised it, on
/// the releasing group to its owner, as the file among `parents` that mounts
/// it names them, or on `programme` to its programme manager. The first that
/// holds is the decision. Then `systems`, the drive's systems' file: when it
/// opens through the chain and is marked so its change rows are signed with
/// it, its first `reallocate` row naming the objection and its answer, made
/// by the system engineer or their deputy. None, undecided.
pub fn decisions(
    objections: &[Objection],
    issues: &[File],
    systems: Option<&File>,
    parents: &[&Registry],
    programme: &Registry,
) -> Vec<Decided> {
    // The rows a signature covers, of a file that opens through the chain:
    // never a row added since it was signed, nor one from before F6-d.
    let rows: &[Change] = systems
        .filter(|f| {
            f.meta.get(chain::SIGNED_ROWS).map(String::as_str) == Some(chain::ROWS_SIGNED)
                && chain::open_systems(f, programme).is_ok()
        })
        .map_or(&[], |f| f.changes.as_slice());
    objections
        .iter()
        .map(|o| {
            let id = o.id();
            let decision = issues.iter().find_map(|f| {
                // An issue file as every reader holds it (`check_issue`),
                // then a decision as only the system engineer makes one.
                chain::check_issue(f, &[programme]).ok()?;
                let [i] = f.issues.as_slice() else {
                    return None;
                };
                let meta = |k: &str| f.meta.get(k).map_or("", String::as_str);
                if f.kind().ok() != Some(Kind::Issue)
                    || meta("number") != i.number.to_string()
                    || meta("group_id") != i.group_id
                    || i.place != id
                    || i.evidence != format!("answer {}", o.answer)
                {
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
                // The programme manager, as `decision_issue` addresses them:
                // never their deputy.
                let to_pm = programme.file.people.iter().any(|p| {
                    p.name == i.addressed_to
                        && p.role == "programme manager"
                        && p.deputy_for.is_empty()
                });
                (i.group_id == "programme" && to_pm).then(|| Decision::ReleaseAnyway {
                    issue: i.number,
                    by: i.raised_by.clone(),
                })
            });
            let decision = decision.or_else(|| {
                rows.iter()
                    .find(|c| {
                        reallocates(c, o)
                            && !crate::checks::is_assistant(&c.who)
                            && programme.acts_as(&c.who, "system engineer")
                    })
                    .map(|c| Decision::Reallocated {
                        by: c.who.clone(),
                        at: c.at.clone(),
                    })
            });
            Decided {
                objection: o.clone(),
                decision,
            }
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

/// An objection and what was decided of it, as [`decisions`] read it: made
/// nowhere else, so what a release is given was read from signed files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decided {
    objection: Objection,
    decision: Option<Decision>,
}

impl Decided {
    pub fn objection(&self) -> &Objection {
        &self.objection
    }

    pub fn decision(&self) -> Option<&Decision> {
        self.decision.as_ref()
    }
}

/// What the previews ask of the design `version`, released after `previous`
/// from today's design holding `holding` (each group's release, by its group
/// and version): refused, by name, while a preview on `previous` of a
/// release today's design holds is unanswered or answered by a file that
/// does not count, while `unread` says a preview that may be such a one does
/// not open, while an objection to one has no decision, or while one decided
/// *next version* is of a release today's design still holds. A preview of
/// another release, or on another design, is stale and asks nothing
/// (question 46). Gives the red marks the objections released anyway are,
/// each raised by its programme manager's issue, and the rows of
/// `objections.csv`, one for every objection decided.
pub fn asked_of(
    previews: &[Standing],
    unread: &[String],
    decided: &[Decided],
    previous: Option<&str>,
    holding: &[(&str, &str)],
) -> Result<(Vec<Mark>, Vec<Vec<String>>), Error> {
    let held = |g: &str, v: &str| holding.iter().any(|(hg, hv)| *hg == g && *hv == v);
    let live: Vec<&Standing> = previews
        .iter()
        .filter(|p| Some(p.based_on.as_str()) == previous && held(&p.group, &p.version))
        .collect();
    let mut wanting: Vec<String> = unread
        .iter()
        .map(|why| format!("a preview that may be one does not open: {why}"))
        .collect();
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
    let mut again = Vec::new();
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
                again.push(format!(
                    "{} {} is to be released again, as issue {issue} asks for {g}'s objection, and today's design still holds it",
                    p.group, p.version
                ));
                continue;
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
    if !open.is_empty() || !again.is_empty() {
        let mut said = Vec::new();
        if !open.is_empty() {
            said.push(format!(
                "every objection is decided before the design is released (W7), and these are not: {}. Ask for its next version, release anyway, or change an allocation",
                open.join("; ")
            ));
        }
        if !again.is_empty() {
            said.push(format!(
                "the design is not released until each next version asked for replaces the release objected to: {}",
                again.join("; ")
            ));
        }
        return Err(refused(said.join(". ")));
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
