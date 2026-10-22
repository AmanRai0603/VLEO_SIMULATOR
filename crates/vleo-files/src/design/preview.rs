//! A preview, and the answers to it (docs/briefs/F6.md, F6-c items 3 and 4;
//! W7; docs/OPERATING_1_0.md, section 10).
//!
//! When a group's release in today's design is not the one in the design
//! released now, and taking it in moves a value outside its group, the
//! system engineer, or their deputy, writes a **preview** of it ([`preview`]):
//! the release whole, `based_on` the design released now, `moved.csv`, every
//! value outside the group that moved, and `named.csv`, every subsystem's
//! group a value moved in, whose owner answers. What moved is the engine's,
//! run on two graphs it built itself (the server's `POST /v1/design/preview`); here
//! nothing is computed, only held and checked. A move in the systems' or the
//! programme's branch is shown and not answered: the system engineer decides,
//! and the programme manager owns the decision (F6's question 45).
//!
//! The preview is at `integration/<g>-<v>/preview-<g>-<v>-on-<design>.vleo`
//! ([`chain::preview_path`]), beside the answers to the release, never
//! written over; on a later design the release is previewed again under a
//! new name (question 53), and on the same design, once the one there no
//! longer opens, at `...-on-<design>-<n>.vleo` from 2
//! ([`chain::preview_path_n`]). It is read back by [`open_preview`], wherever one
//! is relied on: `chain::check_preview`, and the release it holds read as
//! the release it says it is.
//!
//! **The answers** ([`answers`]): for each group named, *fine*, *object,
//! because…*, *unanswered*, or *refused, why*, each read through
//! `chain::check_answer` and held to the release the preview holds, by its
//! fingerprint. An answer is to the release, not to the preview, so one
//! given on an earlier design's preview still answers it.
//!
//! Not here: the decision on an objection, and the release refused while a
//! preview is unanswered (F6-c3b).

#[cfg(not(target_arch = "wasm32"))]
use super::{read_bytes, Opened};
use crate::answer::{self, Said};
use crate::chain::{self, Registry};
#[cfg(not(target_arch = "wasm32"))]
use crate::chain::{FILE, PREVIEW_MOVED, PREVIEW_NAMED};
use crate::error::{Error, ErrorKind};
#[cfg(not(target_arch = "wasm32"))]
use crate::keys::{hex, sha256, SigningKey};
#[cfg(not(target_arch = "wasm32"))]
use crate::meta::Kind;
use crate::model::File;
#[cfg(not(target_arch = "wasm32"))]
use crate::model::{Media, Tbl};

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
fn refused(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Malformed, message)
}

/// A value outside the release's group that moved: its id, `<node>` or
/// `<node>.<port>`, its node and the group it hangs under, and its answer in
/// the design released now (`before`) and with the release in it (`after`),
/// `None` where it refused, with its node's reason.
#[derive(Clone, Debug, PartialEq)]
pub struct Move {
    pub value: String,
    pub node: String,
    pub group: String,
    pub before: Option<f64>,
    pub after: Option<f64>,
    pub before_why: Option<String>,
    pub after_why: Option<String>,
}

/// A preview read back: the file, what it previews, the design it is on,
/// the release it holds, what moved, and who answers.
#[derive(Clone, Debug)]
pub struct OpenedPreview {
    pub file: File,
    pub group: String,
    pub version: String,
    pub based_on: String,
    pub writer: String,
    /// The fingerprint of the release held, which an answer names.
    pub fingerprint: String,
    pub release: super::Back,
    pub moved: Vec<Move>,
    pub named: Vec<String>,
}

/// What a group named in a preview has said of the release.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answered {
    Fine,
    Object(String),
    Unanswered,
    /// An answer that does not count, and why.
    Refused(String),
}

impl Answered {
    pub fn name(&self) -> &'static str {
        match self {
            Answered::Fine => "fine",
            Answered::Object(_) => "object",
            Answered::Unanswered => "unanswered",
            Answered::Refused(_) => "refused",
        }
    }
}

/// The groups among those `moved` names whose owners answer `group`'s
/// release: each a subsystem's group mounted through `parents` (the
/// programme's and the systems' files), owned by a subsystem engineer the
/// programme's file registers who does not own `group` too, since a
/// release's owner does not answer it for another group
/// (`chain::check_answer`). The systems' and the programme's branches, a
/// value at the design's root, and a group nothing mounts are shown in
/// `moved.csv` and not named.
pub fn named(
    moved: &[Move],
    group: &str,
    parents: &[&Registry],
    programme: &Registry,
) -> Vec<String> {
    let releaser = chain::mounted_by(group, parents).ok().map(|(_, o)| o);
    let mut out: Vec<String> = Vec::new();
    for m in moved {
        let g = m.group.as_str();
        if out.iter().any(|n| n == g)
            || g == group
            || ["systems", "programme"].contains(&g)
            || !crate::breakdown::valid_id(g)
        {
            continue;
        }
        let Ok((_, owner)) = chain::mounted_by(g, parents) else {
            continue;
        };
        let subsystem = programme
            .file
            .people
            .iter()
            .any(|p| p.name == owner && p.role == "subsystem engineer");
        if subsystem && releaser != Some(owner) {
            out.push(g.to_string());
        }
    }
    out.sort();
    out
}

/// The cell an answer is written as: the engine's own number, written so it
/// reads back as the same bits, or nothing where it refused.
#[cfg(not(target_arch = "wasm32"))]
fn cell(x: Option<f64>) -> String {
    x.map_or(String::new(), |x| format!("{x:?}"))
}

/// Whether `group`'s release `version`, by its `fingerprint`, has anything
/// to preview on `based_on`, the design released now: refused by name when
/// the design holds it already (F6's question 43).
#[cfg(not(target_arch = "wasm32"))]
pub fn to_preview(
    based_on: &Opened,
    group: &str,
    version: &str,
    fingerprint: &str,
) -> Result<(), Error> {
    if based_on
        .releases
        .iter()
        .any(|r| r.group == group && r.fingerprint == fingerprint)
    {
        return Err(refused(format!(
            "design {} holds {group} {version} already: there is nothing to preview",
            based_on.version
        )));
    }
    Ok(())
}

/// The preview of the release `release`, the bytes of the group's release
/// file as the drive holds it at `release.path`, on `based_on`, the design
/// released now read back, written by `by` with `key` at `at`: holding the
/// release whole, every value of `moved` and the groups whose owners answer
/// ([`named`], through `parents`). Refused unless the release is sealed
/// through the chain, is one the design does not already hold, it moves a value outside its group, and
/// `by` is the system engineer or their deputy with a key the programme's
/// file counts then; held to `chain::check_preview` as it will be read.
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
pub fn preview(
    release: &super::Whole,
    based_on: &Opened,
    moved: &[Move],
    parents: &[&Registry],
    by: &str,
    key: &SigningKey,
    at: &str,
    app: &str,
    programme: &Registry,
) -> Result<File, Error> {
    let how = crate::upgrade::Upgrade { app, at };
    let file = read_bytes(release.bytes, &how)
        .map_err(|e| refused(format!("{} does not read: {}", release.path, e.message())))?;
    if file.kind()? != Kind::GroupRelease {
        return Err(refused(format!(
            "a preview is of a group's release, and {} is a {}",
            release.path,
            file.kind()?
        )));
    }
    let get = |k: &str| file.meta.get(k).cloned().unwrap_or_default();
    let (group, version) = (get("group_id"), get("version"));
    let fingerprint = fingerprint_of(&file)?;
    let at_path = format!("groups/{group}/releases/{group}-{version}.vleo");
    if release.path != at_path {
        return Err(refused(format!(
            "{} is {group} {version}, which the drive keeps as {at_path}",
            release.path
        )));
    }
    sealed(&file, &group, &version, parents, programme)?;
    to_preview(based_on, &group, &version, &fingerprint)?;
    if moved.is_empty() {
        return Err(refused(format!(
            "{group} {version} moves nothing outside {group} on design {}: it needs no preview",
            based_on.version
        )));
    }
    if crate::checks::is_assistant(by) {
        return Err(refused(format!(
            "{by} is an assistant's name: a preview is written by a person"
        )));
    }
    programme.key(by, &key.public().to_hex(), at)?;
    if !programme.acts_as(by, "system engineer") {
        return Err(Error::new(
            ErrorKind::Signature,
            format!(
                "{by} may not write a preview: the system engineer, or their deputy, writes one (W7)"
            ),
        ));
    }
    let mut f = File::new(Kind::Preview, app);
    for (k, v) in [
        ("group_id", group.as_str()),
        ("version", version.as_str()),
        ("based_on", based_on.version.as_str()),
        ("writer", by),
    ] {
        f.meta.insert(k.into(), v.to_string());
    }
    f.media.push(Media {
        scope: FILE.into(),
        path: at_path,
        media_type: "application/vnd.vleo".into(),
        sha256: hex(&sha256(release.bytes)),
        bytes: release.bytes.to_vec(),
    });
    let rows: Vec<Vec<String>> = moved
        .iter()
        .map(|m| {
            vec![
                m.value.clone(),
                m.node.clone(),
                m.group.clone(),
                cell(m.before),
                cell(m.after),
                m.before_why.clone().unwrap_or_default(),
                m.after_why.clone().unwrap_or_default(),
            ]
        })
        .collect();
    f.tables.push(Tbl {
        scope: FILE.into(),
        path: "moved.csv".into(),
        csv: crate::csv::write(PREVIEW_MOVED, &rows),
    });
    let names: Vec<Vec<String>> = named(moved, &group, parents, programme)
        .into_iter()
        .map(|g| vec![g])
        .collect();
    f.tables.push(Tbl {
        scope: FILE.into(),
        path: "named.csv".into(),
        csv: crate::csv::write(PREVIEW_NAMED, &names),
    });
    f.check_meta()?;
    let row = chain::sign(&f, FILE, by, key, at, "ok", "")?;
    f.signatures.push(row);
    chain::check_preview(&f, programme).map_err(|e| {
        refused(format!(
            "the preview would not open through the chain as it will be read, so it is not written: {}",
            e.message()
        ))
    })?;
    Ok(f)
}

/// Whether `file`, `group`'s release `version`, is sealed through the
/// chain: its owner's seal, as the file among `parents` that mounts the group
/// names its owner (`chain::seal_of`); or, from before 1.0, its anchor in the
/// programme's file. A release nobody sealed is never previewed.
#[cfg(not(target_arch = "wasm32"))]
fn sealed(
    file: &File,
    group: &str,
    version: &str,
    parents: &[&Registry],
    programme: &Registry,
) -> Result<(), Error> {
    let (mounted_by, _) = chain::mounted_by(group, parents)?;
    if file.meta.get("upgraded_from").map(String::as_str) == Some(crate::upgrade::FROM_FORMAT_1) {
        let checked = chain::check_release_in(file, mounted_by, programme)?;
        if let Some(why) = checked.refused.first() {
            return Err(refused(format!("{group} {version} is not sealed: {why}")));
        }
        return Ok(());
    }
    chain::seal_of(file, mounted_by, programme).map(|_| ())
}

/// The fingerprint of a group's release, as its seal names it: a release of
/// the one schema gives its own digest; one from before 1.0 was held to its
/// folder's when it was upgraded, and says it.
#[cfg(not(target_arch = "wasm32"))]
fn fingerprint_of(file: &File) -> Result<String, Error> {
    let said = file.meta.get("fingerprint").cloned().unwrap_or_default();
    let before_1_0 =
        file.meta.get("upgraded_from").map(String::as_str) == Some(crate::upgrade::FROM_FORMAT_1);
    if before_1_0 && !said.is_empty() {
        return Ok(said);
    }
    let digest = chain::digest(file, FILE)?;
    if said != digest {
        return Err(refused(format!(
            "{} {} says its fingerprint is {said:?}, and what it holds gives {digest}: it is not the release its seal names",
            file.meta.get("group_id").map_or("", String::as_str),
            file.meta.get("version").map_or("", String::as_str),
        )));
    }
    Ok(digest)
}

/// The preview `f` read back, as it is read wherever one is relied on: it
/// checks through `programme`'s chain (`chain::check_preview`), and the
/// release it holds reads as a group's release, the group and version the
/// preview says, whose fingerprint is its content's, sealed through the
/// chain of `parents` (the programme's and the systems' files). Anything
/// else is refused by name.
#[cfg(not(target_arch = "wasm32"))]
pub fn open_preview(
    f: &File,
    parents: &[&Registry],
    programme: &Registry,
    app: &str,
) -> Result<OpenedPreview, Error> {
    chain::check_preview(f, programme)?;
    let get = |k: &str| f.meta.get(k).cloned().unwrap_or_default();
    let (group, version, writer) = (get("group_id"), get("version"), get("writer"));
    let named_as = |what: String| refused(format!("the preview of {group} {version} {what}"));
    // Read as of the signature that counts, never a row beside it.
    let (counts, _) = chain::counted(f, FILE, programme, |name| {
        name == writer && programme.acts_as(name, "system engineer")
    });
    let at = counts
        .first()
        .map_or(String::new(), |s| s.signed_at.clone());
    let how = crate::upgrade::Upgrade { app, at: &at };
    let m = &f.media[0];
    let file = read_bytes(&m.bytes, &how).map_err(|e| {
        named_as(format!(
            "holds {}, which does not read: {}",
            m.path,
            e.message()
        ))
    })?;
    let said = |k: &str| file.meta.get(k).cloned().unwrap_or_default();
    if file.kind().ok() != Some(Kind::GroupRelease)
        || said("group_id") != group
        || said("version") != version
    {
        return Err(named_as(format!(
            "holds {}, which is not {group}'s release {version}",
            m.path
        )));
    }
    let fingerprint = fingerprint_of(&file)
        .map_err(|e| named_as(format!("holds {}: {}", m.path, e.message())))?;
    sealed(&file, &group, &version, parents, programme)
        .map_err(|e| named_as(format!("holds {}: {}", m.path, e.message())))?;
    let table = |path: &str| {
        crate::csv::read(
            &f.tables
                .iter()
                .find(|t| t.path == path)
                .map_or(String::new(), |t| t.csv.clone()),
        )
        .rows
    };
    let number = |c: &str| (!c.is_empty()).then(|| c.parse::<f64>().ok()).flatten();
    let why = |c: &str| (!c.is_empty()).then(|| c.to_string());
    let moved = table("moved.csv")
        .into_iter()
        .map(|r| Move {
            value: r[0].clone(),
            node: r[1].clone(),
            group: r[2].clone(),
            before: number(&r[3]),
            after: number(&r[4]),
            before_why: why(&r[5]),
            after_why: why(&r[6]),
        })
        .collect();
    let named = table("named.csv")
        .into_iter()
        .map(|r| r[0].clone())
        .collect();
    Ok(OpenedPreview {
        group,
        version,
        based_on: get("based_on"),
        writer: get("writer"),
        fingerprint,
        release: super::Back {
            path: m.path.clone(),
            bytes: m.bytes.clone(),
            file,
        },
        moved,
        named,
        file: f.clone(),
    })
}

/// What each group the preview `p` names has said of its release, in the
/// order it names them: the answer `read` gives at its place
/// (`answer::path_of`), `None` where there is none, read through
/// `chain::check_answer` with the file among `parents` that mounts the
/// answering group, and held to the release the preview holds, by group,
/// version and fingerprint. An answer that does not count is refused, why
/// said, never taken for unanswered.
pub fn answers(
    p: &OpenedPreview,
    read: &dyn Fn(&str) -> Option<Result<File, String>>,
    parents: &[&Registry],
    programme: &Registry,
) -> Vec<(String, Answered)> {
    p.named
        .iter()
        .map(|g| {
            let path = answer::path_of(&p.group, &p.version, g);
            let state = match read(&path) {
                None => Answered::Unanswered,
                Some(Err(why)) => Answered::Refused(format!("{path} does not read: {why}")),
                Some(Ok(f)) => answered(&f, p, g, parents, programme),
            };
            (g.clone(), state)
        })
        .collect()
}

fn answered(
    f: &File,
    p: &OpenedPreview,
    g: &str,
    parents: &[&Registry],
    programme: &Registry,
) -> Answered {
    let said = |k: &str| f.meta.get(k).map_or("", String::as_str);
    if said("group_id") != p.group || said("version") != p.version || said("answering_group") != g {
        return Answered::Refused(format!(
            "it says it is {}'s answer to {} {}, not {g}'s to {} {}",
            said("answering_group"),
            said("group_id"),
            said("version"),
            p.group,
            p.version
        ));
    }
    if said("answers") != p.fingerprint {
        return Answered::Refused(format!(
            "it answers a release whose fingerprint is {:?}, and the preview holds {} {} as {}",
            said("answers"),
            p.group,
            p.version,
            p.fingerprint
        ));
    }
    let checked = chain::mounted_by(g, parents)
        .and_then(|(mb, _)| chain::check_answer(f, mb, programme))
        .and_then(|()| answer::said(f));
    match checked {
        Ok(Said::Fine) => Answered::Fine,
        Ok(Said::Object(because)) => Answered::Object(because),
        Err(e) => Answered::Refused(e.message().to_string()),
    }
}
