//! The person's own work put back on the shared drive from their working copy
//! (docs/briefs/F3.md, F3-c; `vleo_files::put_back`).
//!
//! `POST /v1/put-back` answers the report first: every file changed in the
//! copy, whether it goes back as the open key's person's work, and why, or
//! why not. Nothing is written. Asked again with `files`, the files to put
//! back each as the report gave it (its digest, a space, its path, one to a
//! line), and those exactly the ones the report gives as it is made again
//! under the drive's lock, they are written on the drive. The chain is
//! opened from the drive's own files, never the copy's, and the person's key
//! must still count at the moment the work goes back: a key revoked on the
//! drive since signs nothing that goes back.

use super::*;
use vleo_files::put_back::{put_back, Chain};

/// `POST /v1/put-back`.
pub(super) fn route(params: &str, ctx: &Ctx) -> String {
    match answer(params, ctx) {
        Ok(j) => j,
        Err(why) => me::refusal(&why),
    }
}

fn answer(params: &str, ctx: &Ctx) -> Result<String, String> {
    me::carries_session(params)?;
    let Some(by) = me::with_key(|_, me| me.name.clone()) else {
        return Err("no key is open: open yours, and what you signed goes back as yours".into());
    };
    let copy = copy::dir(ctx);
    let Some(record) = copy::record(&copy)? else {
        return Err(
            "there is no copy of the database here to put back from: copy it from the drive first"
                .into(),
        );
    };
    let drive = PathBuf::from(&record.drive);
    // The chain, opened from the drive's own files: the library opens it
    // again under the drive's lock before it writes.
    let open = || -> Result<Chain, vleo_files::error::Error> {
        let fail =
            |m: String| vleo_files::error::Error::new(vleo_files::error::ErrorKind::Signature, m);
        match me::registries(&drive, &copy, "") {
            Ok(c) => Ok(Chain {
                anchor: c.anchor,
                programme: c.programme,
                systems: c.systems,
                groups: c.groups,
                left_out: c.left_out,
            }),
            Err(me::NotRead::FirstUse(..)) => Err(fail(
                "this computer has not accepted this drive's anchor: open your key on it first"
                    .into(),
            )),
            Err(me::NotRead::Why(why)) => Err(fail(format!(
                "the drive's files do not open through the chain, so nothing goes back: {why}"
            ))),
        }
    };
    // Each file named as the report gave it: its digest, a space, its path.
    let named: Option<Vec<(String, String)>> = match param(params, "files").map(decode) {
        None => None,
        Some(v) => {
            let mut out = Vec::new();
            for l in v.lines().map(str::trim).filter(|l| !l.is_empty()) {
                let Some((digest, path)) = l.split_once(' ') else {
                    return Err(format!(
                        "{l:?} does not name a file as the report gave it: its digest, a space, and its path"
                    ));
                };
                out.push((path.to_string(), digest.to_string()));
            }
            Some(out)
        }
    };
    let report = put_back(
        &copy,
        &drive,
        &by,
        &results::now_utc(),
        &open,
        named.as_deref(),
    )
    .map_err(|e| e.message().to_string())?;
    let back = report.back();
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.bool_field("written", report.written);
    j.key("recorded").raw("[");
    for (i, p) in report.recorded.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw(&json::string(p));
    }
    j.raw("]");
    j.str_field("by", &by);
    j.str_field("drive", &record.drive);
    j.key("files").raw("[");
    for (i, l) in report.lines.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("path", &l.path);
        j.bool_field("back", l.back);
        j.str_field("why", &l.why);
        j.str_field("digest", &l.digest);
        j.raw("}");
    }
    j.raw("]");
    j.key("back").raw("[");
    for (i, p) in back.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw(&json::string(p));
    }
    j.raw("]");
    let stays = report.lines.len() - back.len();
    j.str_field(
        "message",
        &if report.written {
            format!(
                "{} back on the drive as {by}'s: {}.",
                count(back.len(), "file is", "files are"),
                back.join(", ")
            )
        } else if !report.recorded.is_empty() {
            format!(
                "{} already on the drive as your copy holds it: your copy's record now says so. Nothing else is written.",
                count(report.recorded.len(), "file is", "files are")
            )
        } else if report.lines.is_empty() {
            "Nothing in your copy has changed since it was taken: nothing goes back.".to_string()
        } else {
            format!(
                "{} go back to the drive as {by}'s, and {} in your copy, each with why. Nothing is written until you put back the files named.",
                count(back.len(), "file would", "files would"),
                count(stays, "stays", "stay")
            )
        },
    );
    j.raw("}");
    Ok(j.0)
}

/// `n` and its words: "1 file is", "2 files are".
fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}
