//! A group's answer to another group's release that moves its values
//! (docs/briefs/F5.md, item 4; W7; docs/OPERATING_1_0.md, section 10).
//!
//! When a release moves another group's values, that group's owner answers
//! it, signed: **fine**, or **object, because…**. The answer is a file of its
//! own, a *preview answer*, `<group>-<version>.<answering group>.answer` in
//! the drive's `integration/<group>-<version>/`, never changed once written.
//! It is the answering group's owner's, or their deputy's, with a key the
//! programme's file counts for them, as the file that mounts the answering
//! group names its owner (`chain::check_answer`, the one rule, wherever an
//! answer is read). It names the release it answers by that release's
//! fingerprint, and is to a release that is there: never to a version not
//! sealed yet. The answering group is a subsystem's, owned by a subsystem
//! engineer who does not own the release's group: the system engineer
//! decides an objection, and does not answer. An objection says why; an
//! answer says nothing else.
//!
//! Not here: which groups a release moves, which the preview the system
//! engineer writes says (F6), and the system engineer's decision on an
//! objection (W7, F6).

use crate::chain::{self, Registry, FILE};
use crate::error::{Error, ErrorKind};
use crate::keys::SigningKey;
use crate::meta::{Kind, ReleaseVersion};
use crate::model::{File, Text};

fn refused(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Malformed, message)
}

/// What an answer says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Said {
    Fine,
    /// An objection, and why.
    Object(String),
}

/// The texts an answer keeps what it says in, scoped to the file.
const ANSWER: &str = "answer";
const BECAUSE: &str = "because";

/// Where the answer of `answering` to `group`'s release `version` is, inside
/// a copy or the drive.
pub fn path_of(group: &str, version: &str, answering: &str) -> String {
    format!("integration/{group}-{version}/{group}-{version}.{answering}.answer")
}

/// What `f`, an answer, says, or why it says nothing that is one. It holds
/// its word, and why where it objects, once each, and no other text: an
/// answer saying two things is refused, never read as the first.
pub fn said(f: &File) -> Result<Said, Error> {
    if let Some(t) = f
        .texts
        .iter()
        .find(|t| t.scope != FILE || (t.kind != ANSWER && t.kind != BECAUSE))
    {
        return Err(refused(format!(
            "the answer holds a text {:?} of {}: an answer holds its word and why, and nothing else",
            t.kind, t.scope
        )));
    }
    for k in [ANSWER, BECAUSE] {
        if f.texts.iter().filter(|t| t.kind == k).count() > 1 {
            return Err(refused(format!(
                "the answer says its {k} twice: which one it means cannot be said"
            )));
        }
    }
    let text = |k: &str| {
        f.texts
            .iter()
            .find(|t| t.scope == FILE && t.kind == k)
            .map(|t| t.body.trim().to_string())
    };
    match text(ANSWER).as_deref() {
        Some("fine") => Ok(Said::Fine),
        Some("object") => match text(BECAUSE).filter(|b| !b.is_empty()) {
            Some(b) => Ok(Said::Object(b)),
            None => Err(refused("the answer objects and does not say why")),
        },
        Some(other) => Err(refused(format!(
            "the answer says {other:?}, which is neither fine nor object"
        ))),
        None => Err(refused("the answer says nothing")),
    }
}

/// The answer of the group `answering` to `release`, a group's sealed
/// release, saying `said`, by `by` with `key` at `at`, signed and naming the
/// release by its fingerprint: refused, by name, unless `by` is the
/// answering group's owner or their deputy, as `mounted_by`, the file that
/// mounts it, names its owner, with a key the programme's file counts for
/// them now. A group does not answer its own release, nor does its owner
/// answer it for another group; the systems' branch does not answer; and an
/// objection says why.
#[allow(clippy::too_many_arguments)]
pub fn answer(
    release: &File,
    answering: &str,
    said: &Said,
    by: &str,
    key: &SigningKey,
    at: &str,
    app: &str,
    mounted_by: &Registry,
    programme: &Registry,
) -> Result<File, Error> {
    if release.kind()? != Kind::GroupRelease {
        return Err(refused(format!(
            "an answer is to a group's release, and this is a {}",
            release.kind()?
        )));
    }
    let get = |k: &str| release.meta.get(k).map_or("", String::as_str);
    let (group, version) = (get("group_id"), get("version"));
    let answers = chain::digest(release, FILE)?;
    if get("fingerprint") != answers {
        return Err(refused(format!(
            "{group} {version} says its fingerprint is {:?}, and what it holds gives {answers}: an answer is to the release its seal names",
            get("fingerprint")
        )));
    }
    for id in [group, answering] {
        if !crate::breakdown::valid_id(id) {
            return Err(refused(format!("{id:?} is not a group's id")));
        }
    }
    if group == answering {
        return Err(refused(format!(
            "{answering} does not answer its own release: an answer is another group's"
        )));
    }
    if answering == "systems" {
        return Err(refused(
            "the systems' branch decides an objection to a release, and does not answer one: a subsystem's group answers",
        ));
    }
    if ReleaseVersion::parse(version).is_none() {
        return Err(refused(format!(
            "{version:?} is not a release's version: two whole numbers, major.minor"
        )));
    }
    if crate::checks::is_assistant(by) {
        return Err(refused(format!(
            "{by} is an assistant's name: an answer is a person's"
        )));
    }
    if let Said::Object(because) = said {
        if because.trim().is_empty() {
            return Err(refused(
                "an objection says why: object, because… — and this one says nothing",
            ));
        }
    }
    programme.key(by, &key.public().to_hex(), at)?;
    let owner = chain::owner_of(mounted_by, answering).ok_or_else(|| {
        refused(format!(
            "{} mounts no group {answering:?}, so nobody answers for it",
            mounted_by.from
        ))
    })?;
    if !programme.acts_for(by, owner) {
        return Err(refused(format!(
            "{by} may not answer for {answering}: its owner {owner}, or their deputy, answers"
        )));
    }
    let mut f = File::new(Kind::PreviewAnswer, app);
    for (k, v) in [
        ("group_id", group),
        ("version", version),
        ("answers", answers.as_str()),
        ("answering_group", answering),
        ("writer", by),
    ] {
        f.meta.insert(k.into(), v.to_string());
    }
    let (word, because) = match said {
        Said::Fine => ("fine", None),
        Said::Object(b) => ("object", Some(b.trim().to_string())),
    };
    f.texts.push(Text {
        scope: FILE.into(),
        kind: ANSWER.into(),
        body: word.into(),
    });
    if let Some(b) = because {
        f.texts.push(Text {
            scope: FILE.into(),
            kind: BECAUSE.into(),
            body: b,
        });
    }
    f.check_meta()?;
    // Its issue and change rows are signed with it (F6-d).
    chain::sign_rows(&mut f);
    let row = chain::sign(&f, FILE, by, key, at, "ok", "")?;
    f.signatures.push(row);
    chain::check_answer(&f, mounted_by, programme)?;
    Ok(f)
}

/// The answer made as [`answer`] makes it, to `group`'s release `version` as
/// the copy holds it, written into the copy at its place ([`path_of`]),
/// under the copy's lock; one already there is never written over, and a
/// release the copy does not hold is not answered. Gives its path inside
/// the copy.
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
pub fn answer_in_place(
    copy: &std::path::Path,
    group: &str,
    version: &str,
    answering: &str,
    said: &Said,
    by: &str,
    key: &SigningKey,
    at: &str,
    app: &str,
    mounted_by: &Registry,
    programme: &Registry,
) -> Result<String, Error> {
    let _held = crate::save::lock(copy)?;
    for id in [group, answering] {
        if !crate::breakdown::valid_id(id) {
            return Err(refused(format!("{id:?} is not a group's id")));
        }
    }
    let at_release = copy
        .join("groups")
        .join(group)
        .join("releases")
        .join(format!("{group}-{version}.vleo"));
    if !at_release.is_file() {
        return Err(refused(format!(
            "your copy holds no release {group} {version}: an answer is to a release that is there"
        )));
    }
    let how = crate::upgrade::Upgrade { app, at };
    let release = crate::sqlite::open(&at_release, &how)?.file;
    let f = answer(
        &release, answering, said, by, key, at, app, mounted_by, programme,
    )?;
    let rel = path_of(group, version, answering);
    let at_file = copy.join(&rel);
    if at_file.exists() {
        return Err(refused(format!(
            "{rel} is there already: an answer is never written over"
        )));
    }
    if let Some(dir) = at_file.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", dir.display())))?;
    }
    crate::sqlite::write(&f, &at_file)?;
    Ok(rel)
}
