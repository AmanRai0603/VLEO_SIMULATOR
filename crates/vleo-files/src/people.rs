//! The people a file registers, changed at the valve that owns them.
//!
//! docs/OPERATING_1_0.md, section 14, and docs/briefs/F1.md, PR 2:
//!
//! - **The programme manager** registers, replaces and revokes the keys in the
//!   programme's file: the system engineer's, every subsystem engineer's and
//!   every deputy's.
//! - **A group's owner**, or their deputy, does the same for the node engineers
//!   in their group file, and for nobody else.
//! - **A person's own key** is never changed by themselves: whoever registered
//!   them replaces it. The programme manager's is the anchor, and changes with
//!   a note signed by the old key, not here.
//!
//! A change gives the file back re-signed by whoever made it, and checked
//! through the chain as it will be read. A change that would leave the file
//! refused when it is next opened is refused now, and nothing is given back.

use crate::chain::{self, Registry, FILE};
use crate::error::{Error, ErrorKind};
use crate::keys::{PublicKey, SigningKey};
use crate::model::{File, Person, PersonKey};

fn refused(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Signature, message)
}

/// The roles each kind of file registers.
const IN_THE_PROGRAMME: &[&str] = &["programme manager", "system engineer", "subsystem engineer"];
const IN_A_GROUP: &[&str] = &["node engineer"];

/// Where a file sits in the chain, and so who may change whom it registers.
pub enum Valve<'a> {
    /// The programme's file, opened against the anchor START HERE holds.
    Programme { anchor: &'a str },
    /// A group file, the systems' among them, through the file that mounts it
    /// and the programme's file that registers its owner.
    Group {
        mounted_by: &'a Registry,
        programme: &'a Registry,
    },
}

/// One change to the people a file registers.
pub enum Change {
    /// A person not yet registered here, with their first key.
    Register { person: Person, public_key: String },
    /// A registered person's key: their current one revoked from the change,
    /// and this one counted from it.
    Replace { person: String, public_key: String },
    /// One key of a registered person, revoked from the change: a signature
    /// made with it from then does not check; one made before still does.
    Revoke { person: String, public_key: String },
}

impl Valve<'_> {
    /// `file` opened as the chain reads it.
    fn open(&self, file: &File) -> Result<Registry, Error> {
        match self {
            Valve::Programme { anchor } => chain::open_programme(file.clone(), anchor),
            Valve::Group {
                mounted_by,
                programme,
            } => chain::open_group_in(file, mounted_by, programme),
        }
    }
}

/// `file` with `change` made by `by`, signing with `key` at `at`, if the
/// valve lets them: the file opens through the chain as it is, `by` holds the
/// valve with a key registered for them at `at`, the change is one the valve
/// makes, and the file opens again afterwards, signed by `by`.
pub fn change(
    file: &File,
    valve: &Valve,
    by: &str,
    key: &SigningKey,
    at: &str,
    change: &Change,
) -> Result<File, Error> {
    let when = chain::instant(at, "the moment of the change")?;
    let now = valve.open(file)?;
    let mine = key.public().to_hex();
    let (roles, holder) = match valve {
        Valve::Programme { .. } => {
            now.key(by, &mine, at)?;
            if !now.acts_as(by, "programme manager") {
                return Err(refused(format!(
                    "{by} may not change whom the programme's file registers: the programme manager does"
                )));
            }
            (IN_THE_PROGRAMME, "the programme manager")
        }
        Valve::Group {
            mounted_by,
            programme,
        } => {
            let id = file.meta.get("group_id").cloned().unwrap_or_default();
            programme.key(by, &mine, at)?;
            let owner = chain::owner_of(mounted_by, &id).unwrap_or_default();
            if !programme.acts_for(by, owner) {
                return Err(refused(format!(
                    "{by} may not change whom {id}'s group file registers: its owner {owner}, or their deputy, does"
                )));
            }
            if let Change::Register { person, .. } = change {
                if programme.file.people.iter().any(|p| p.name == person.name) {
                    return Err(refused(format!(
                        "{} is registered in the programme's file: a group file registers only its node engineers, and nobody twice",
                        person.name
                    )));
                }
            }
            (IN_A_GROUP, "their subsystem engineer")
        }
    };
    let mut out = file.clone();
    match change {
        Change::Register { person, public_key } => {
            if person.name.trim().is_empty() {
                return Err(refused("a person is registered by their name"));
            }
            if out.people.iter().any(|p| p.name == person.name) {
                return Err(refused(format!(
                    "{} is registered in {} already: replace their key, or revoke it",
                    person.name, now.from
                )));
            }
            if !roles.contains(&person.role.as_str()) {
                return Err(refused(format!(
                    "{} registers {}, not a {}",
                    now.from,
                    roles.join(", "),
                    person.role
                )));
            }
            if !person.deputy_for.is_empty()
                && !out
                    .people
                    .iter()
                    .any(|p| p.name == person.deputy_for && p.role == person.role)
            {
                return Err(refused(format!(
                    "{} is registered as deputy for {}, who is not a {} in {}: a deputy holds the role of whom they stand in for",
                    person.name, person.deputy_for, person.role, now.from
                )));
            }
            new_key(&out, &person.name, public_key, &now.from)?;
            out.people.push(person.clone());
            out.keys.push(PersonKey {
                person: person.name.clone(),
                public_key: public_key.clone(),
                registered_at: at.to_string(),
                registered_by: by.to_string(),
                revoked_from: String::new(),
            });
        }
        Change::Replace { person, public_key } => {
            not_mine(person, by, holder)?;
            registered(&out, person, &now.from)?;
            new_key(&out, person, public_key, &now.from)?;
            for k in out.keys.iter_mut().filter(|k| &k.person == person) {
                if !chain::revoked_by(k, when)? {
                    k.revoked_from = at.to_string();
                }
            }
            out.keys.push(PersonKey {
                person: person.clone(),
                public_key: public_key.clone(),
                registered_at: at.to_string(),
                registered_by: by.to_string(),
                revoked_from: String::new(),
            });
        }
        Change::Revoke { person, public_key } => {
            not_mine(person, by, holder)?;
            registered(&out, person, &now.from)?;
            let k = out
                .keys
                .iter_mut()
                .find(|k| &k.person == person && &k.public_key == public_key)
                .ok_or_else(|| {
                    refused(format!("{} registers no such key for {person}", now.from))
                })?;
            if chain::revoked_by(k, when)? {
                return Err(refused(format!(
                    "{person}'s key is revoked from {} already",
                    k.revoked_from
                )));
            }
            k.revoked_from = at.to_string();
        }
    }
    // The file's signatures were made over what it was; the change is signed
    // afresh by whoever made it, and the file must open as it will be read.
    out.signatures.retain(|s| s.scope != FILE);
    let row = chain::sign(&out, FILE, by, key, at, "ok", "")?;
    out.signatures.push(row);
    valve.open(&out).map_err(|e| {
        refused(format!(
            "the change would leave {} refused when it is next opened, so it is not made: {}",
            now.from,
            e.message()
        ))
    })?;
    Ok(out)
}

fn not_mine(person: &str, by: &str, holder: &str) -> Result<(), Error> {
    if person == by {
        return Err(refused(format!(
            "{by} may not change their own key: {holder} does. A programme manager's key is the anchor, and changes with a note signed by the old key"
        )));
    }
    Ok(())
}

fn registered(f: &File, person: &str, from: &str) -> Result<(), Error> {
    if f.people.iter().any(|p| p.name == person) {
        Ok(())
    } else {
        Err(refused(format!("{person} is not registered in {from}")))
    }
}

/// A key that may be registered for `person`: a public key, and nobody's here
/// already. One key says one person.
fn new_key(f: &File, person: &str, public_key: &str, from: &str) -> Result<(), Error> {
    PublicKey::from_hex(public_key)?;
    if let Some(k) = f.keys.iter().find(|k| k.public_key == public_key) {
        return Err(refused(format!(
            "this key is registered in {from} for {} already: one key says one person, so {person} brings their own",
            k.person
        )));
    }
    Ok(())
}
