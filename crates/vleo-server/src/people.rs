//! Whom a file registers, read and changed (docs/briefs/F1.md, PR 2).
//!
//! `GET /v1/people` reads the people a group file registers and their keys.
//! The three changes (register, replace, revoke) are made through the library
//! (`vleo_files::people::change`) at the valve that owns the file, signed with
//! the open key, and written into the person's working copy only: the drive
//! is never written here. Each one carries this launch's word; the file is
//! read once and written whole, and never over one that changed in the copy
//! since it was read.

use super::*;
use vleo_files::chain::{self, Registry};
use vleo_files::copied;
use vleo_files::model::{File, Person};
use vleo_files::people::{self as valve_people, Change, Valve};
use vleo_files::sqlite;

/// A group's id as a folder holds it: letters, digits and `_`.
fn group_id(params: &str) -> Result<String, String> {
    let id = param(params, "file").map(decode).unwrap_or_default();
    if id.is_empty() || !id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') {
        return Err(format!(
            "name the file by its group, such as programme or l3_solar, not {id:?}"
        ));
    }
    Ok(id)
}

fn group_path(database: &Path, id: &str) -> PathBuf {
    database
        .join("groups")
        .join(id)
        .join(format!("{id}.vgroup"))
}

/// `GET /v1/people?file=<group>`: the people a group file registers, each with
/// their keys, from the copy the program reads (`me::database`).
pub(super) fn list(params: &str, ctx: &Ctx) -> String {
    let id = match group_id(params) {
        Ok(id) => id,
        Err(why) => return me::refusal(&why),
    };
    let path = group_path(&me::database(ctx), &id);
    let f = match sqlite::read(&path) {
        Ok(f) => f,
        Err(e) => return me::refusal(&format!("{}: {}", path.display(), e.message())),
    };
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("file", &id);
    j.key("people").raw("[");
    for (i, p) in f.people.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("name", &p.name);
        j.str_field("role", &p.role);
        j.str_field("deputy_for", &p.deputy_for);
        j.key("keys").raw("[");
        for (n, k) in f.keys.iter().filter(|k| k.person == p.name).enumerate() {
            if n > 0 {
                j.raw(",");
            }
            j.raw("{");
            j.str_field("public_key", &k.public_key);
            j.str_field(
                "fingerprint",
                &vleo_files::keys::PublicKey::from_hex(&k.public_key)
                    .map(|p| p.fingerprint())
                    .unwrap_or_default(),
            );
            j.str_field("registered_at", &k.registered_at);
            j.str_field("registered_by", &k.registered_by);
            j.str_field("revoked_from", &k.revoked_from);
            // A key without its holder's proof signs nothing: said, so the
            // valve's holder sees it is one to replace.
            j.bool_field(
                "proof_holds",
                vleo_files::people::proof_holds(&k.person, &k.public_key, &k.proof),
            );
            j.raw("}");
        }
        j.raw("]}");
    }
    j.raw("]}");
    j.0
}

/// The change a request asks for.
fn asked(kind: &str, params: &str) -> Result<Change, String> {
    let p = |k: &str| param(params, k).map(decode).unwrap_or_default();
    let name = p("name");
    if name.trim().is_empty() {
        return Err("name the person the change is for".into());
    }
    Ok(match kind {
        "register" => Change::Register {
            person: Person {
                name,
                role: p("role"),
                deputy_for: p("deputy_for"),
            },
            public_key: p("public_key"),
            proof: p("proof"),
        },
        "replace" => Change::Replace {
            person: name,
            public_key: p("public_key"),
            proof: p("proof"),
        },
        _ => Change::Revoke {
            person: name,
            public_key: p("public_key"),
            from: p("from"),
        },
    })
}

/// `POST /v1/people/<kind>`: a register, replace or revoke in the group file
/// `file` of the working copy, made by whoever's key is open.
pub(super) fn change(kind: &str, params: &str, ctx: &Ctx) -> String {
    match changed(kind, params, ctx) {
        Ok((id, message)) => {
            let mut j = Json::new();
            j.raw("{");
            j.bool_field("ok", true);
            j.str_field("file", &id);
            j.str_field("message", &message);
            j.raw("}");
            j.0
        }
        Err(why) => me::refusal(&why),
    }
}

fn changed(kind: &str, params: &str, ctx: &Ctx) -> Result<(String, String), String> {
    me::carries_session(params)?;
    let copy = copy::dir(ctx);
    if copy::record(&copy)?.is_none() {
        return Err("whom a file registers is changed in your copy of the database: copy it from the drive first".into());
    }
    let id = group_id(params)?;
    let change = asked(kind, params)?;
    // One program at a time writes the copy, and none copies over it meanwhile.
    let _held = copy::lock(&copy)?;
    let c =
        match me::registries(&copy, &copy, "") {
            Ok(c) => c,
            Err(me::NotRead::FirstUse(..)) => return Err(
                "this computer has not accepted this database's anchor: open your key on it first"
                    .into(),
            ),
            Err(me::NotRead::Why(why)) => return Err(why),
        };
    let path = group_path(&copy, &id);
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let opened = copied::digest(&bytes);
    let file: File =
        sqlite::read(&path).map_err(|e| format!("{}: {}", path.display(), e.message()))?;
    let parents: Vec<&Registry> = std::iter::once(&c.programme)
        .chain(c.systems.iter())
        .collect();
    let valve = if id == "programme" {
        Valve::Programme { anchor: &c.anchor }
    } else {
        let (parent, _) = chain::mounted_by(&id, &parents).map_err(|e| e.message().to_string())?;
        Valve::Group {
            mounted_by: parent,
            programme: &c.programme,
        }
    };
    let at = results::now_utc();
    let (out, by) = me::with_key(|key, me| {
        valve_people::change(&file, &valve, &me.name, key, &at, &change)
            .map(|f| (f, me.name.clone()))
            .map_err(|e| e.message().to_string())
    })
    .ok_or("no key is open: open yours, and the change is signed with it")??;
    // Written only over the file as it was read: a second writer on this copy
    // since then is stopped, and nothing is written.
    let now = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    if copied::digest(&now) != opened {
        return Err(format!(
            "{} changed in the copy while the change was made, so nothing is written: read it again and make the change again",
            path.display()
        ));
    }
    sqlite::write(&out, &path).map_err(|e| format!("{}: {}", path.display(), e.message()))?;
    let what = match &change {
        Change::Register { person, .. } => {
            format!("{} is registered as {}", person.name, person.role)
        }
        Change::Replace { person, .. } => format!("{person}'s key is replaced"),
        Change::Revoke { person, from, .. } => format!("{person}'s key is revoked from {from}"),
    };
    Ok((
        id.clone(),
        if id == "systems" {
            format!("{what} in {id}'s file, signed by {by}, in your copy. It goes back to the drive with Put back.")
        } else {
            format!("{what} in {id}'s file, signed by {by}, in your copy. It stays there for now: a group's file does not go back to the drive yet.")
        },
    ))
}
