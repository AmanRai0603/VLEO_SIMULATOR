//! The person's own work put back on the shared drive from their working copy
//! (docs/briefs/F3.md, F3-c; `vleo_files::put_back`).
//!
//! `POST /v1/put-back` answers the report first: every file changed in the
//! copy, whether it goes back as the open key's person's work, and why, or
//! why not. Nothing is written. Asked again with `files`, the files to put
//! back, and those exactly the ones the report gives as it is made again,
//! they are written on the drive. The chain is opened from the drive's own
//! files, never the copy's: a key revoked on the drive since the copy was
//! taken signs nothing that goes back.

use super::*;
use vleo_files::put_back::{put_back, Drive};

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
    let c = match me::registries(&drive, &copy, "") {
        Ok(c) => c,
        Err(me::NotRead::FirstUse(..)) => {
            return Err(
                "this computer has not accepted this drive's anchor: open your key on it first"
                    .into(),
            )
        }
        Err(me::NotRead::Why(why)) => {
            return Err(format!(
                "the drive's files do not open through the chain, so nothing goes back: {why}"
            ))
        }
    };
    let chain = Drive {
        anchor: &c.anchor,
        programme: &c.programme,
        systems: c.systems.as_ref(),
        groups: &c.groups,
        left_out: &c.left_out,
    };
    let named: Option<Vec<String>> = param(params, "files").map(|v| {
        decode(v)
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect()
    });
    let report = put_back(&copy, &drive, &by, &chain, named.as_deref())
        .map_err(|e| e.message().to_string())?;
    let back = report.back();
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.bool_field("written", report.written);
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
